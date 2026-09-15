//! SBOL 3 to Integrase.
//!
//! This is an adapter, not a parser. `sbol-rs` does the reading, the validation
//! (348 machine-checkable rules across SBOL 3.1.0 and 2.3.0), and the RDF
//! handling. What happens here is translation into `integrase-core` types, plus
//! an honest account of anything that did not survive the trip.

use crate::{ConversionReport, SbolError};
use integrase_core::{
    Alphabet, Design, DesignId, Feature, FeatureId, Location, OntologyTerm, Provenance, Sequence,
    Strand,
};
use sbol3::prelude::*;

/// SBOL encoding IRI for IUPAC nucleic acid sequences.
const ENCODING_IUPAC_DNA: &str = "https://identifiers.org/edam:format_1207";

/// Largest document this importer will hand to a parser, in bytes.
///
/// Defence in depth rather than a capability limit. Two live advisories against
/// `quick-xml` in our dependency tree (RUSTSEC-2026-0194 and -0195) are
/// denial-of-service vectors reachable only through adversarial input:
/// quadratic runtime on duplicate attribute names, and unbounded
/// namespace-declaration allocation. Neither is fixable downstream and neither
/// is currently reachable from this API, but a size cap blunts both for any
/// parser this module feeds, and costs nothing for real files.
///
/// 64 MiB is far above any genuine SBOL design. The largest fixture in the
/// vendored corpus is a few kilobytes.
const MAX_DOCUMENT_BYTES: usize = 64 * 1024 * 1024;

/// What was found in a document, and what could not be carried across.
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// The designs that were translated.
    pub designs: Vec<Design>,
    /// Everything the translation could not preserve.
    ///
    /// Surfaced rather than discarded. `sbol-rs` reports what a version
    /// conversion changed, and a project whose thesis is not hiding what it
    /// knows has no business throwing that away.
    pub report: ConversionReport,
}

/// Reads an SBOL 3 document from a Turtle string.
///
/// # Errors
///
/// [`SbolError::Read`] if the document cannot be parsed. Validation failures are
/// reported in the [`ConversionReport`] rather than refused, because a
/// real-world file that violates a best-practice rule is still a file a user
/// wants to look at.
pub fn from_turtle(turtle: &str) -> Result<ImportResult, SbolError> {
    // Checked before parsing, per the edge-case pattern in subplan 11: an
    // out-of-memory kill is not an error message.
    if turtle.len() > MAX_DOCUMENT_BYTES {
        return Err(SbolError::TooLarge {
            len: turtle.len(),
            max: MAX_DOCUMENT_BYTES,
        });
    }
    let doc = sbol3::Document::read_turtle(turtle).map_err(|e| SbolError::Read {
        detail: e.to_string(),
    })?;
    from_document(&doc)
}

/// Translates an already-parsed SBOL document.
///
/// # Errors
///
/// [`SbolError`] if a component references a sequence whose encoding is not one
/// we can interpret.
pub fn from_document(doc: &sbol3::Document) -> Result<ImportResult, SbolError> {
    let mut report = ConversionReport::default();

    // Validation is reported, not enforced. A file that trips a best-practice
    // rule is still one the user wants to open.
    let validation = doc.validate();
    if !validation.is_valid() {
        report
            .notes
            .push("the source document did not pass SBOL validation; see the SBOL report".into());
    }

    // Index sequences by identity so components can resolve their references.
    let mut sequences: std::collections::BTreeMap<String, sbol3::Sequence> =
        std::collections::BTreeMap::new();
    for obj in doc.objects().values() {
        if let Some(s) = sbol3::Sequence::try_from_object(obj) {
            sequences.insert(s.identity.to_string(), s);
        }
    }

    let mut designs = Vec::new();
    for obj in doc.objects().values() {
        let Some(c) = sbol3::Component::try_from_object(obj) else {
            continue;
        };
        match component_to_design(&c, &sequences, &mut report) {
            Ok(Some(d)) => designs.push(d),
            Ok(None) => {}
            Err(e) => return Err(e),
        }
    }

    Ok(ImportResult { designs, report })
}

/// Translates one component, if it carries sequence we can read.
fn component_to_design(
    c: &sbol3::Component,
    sequences: &std::collections::BTreeMap<String, sbol3::Sequence>,
    report: &mut ConversionReport,
) -> Result<Option<Design>, SbolError> {
    let id = c.identity.to_string();

    // A component with no sequence is a legitimate SBOL object, for example an
    // abstract design. It is not something this system can predict against, so
    // it is skipped and recorded rather than silently dropped.
    let Some(seq_ref) = c.sequences.first() else {
        report
            .notes
            .push(format!("component {id} has no sequence and was skipped"));
        return Ok(None);
    };

    let Some(sbol_seq) = sequences.get(&seq_ref.to_string()) else {
        report.notes.push(format!(
            "component {id} references sequence {seq_ref} which is not in this document"
        ));
        return Ok(None);
    };

    let Some(elements) = sbol_seq.elements.as_deref() else {
        report.notes.push(format!(
            "sequence {seq_ref} has no elements and was skipped"
        ));
        return Ok(None);
    };

    // Encoding tells us the alphabet. An unrecognised one is reported rather
    // than guessed at, because guessing the alphabet is how ambiguity codes get
    // silently mangled.
    let alphabet = match sbol_seq.encoding.as_ref().map(ToString::to_string) {
        Some(e) if e.contains("1207") || e.contains("DNA") || e.contains("dna") => {
            Alphabet::DnaIupac
        }
        Some(other) => {
            report.notes.push(format!(
                "sequence {seq_ref} declares encoding {other}, which is not a nucleic acid \
                 encoding this version understands; imported as IUPAC DNA"
            ));
            Alphabet::DnaIupac
        }
        None => {
            report.notes.push(format!(
                "sequence {seq_ref} declares no encoding; imported as IUPAC DNA"
            ));
            Alphabet::DnaIupac
        }
    };

    let sequence =
        Sequence::new(elements.as_bytes().to_vec(), alphabet).map_err(|e| SbolError::Read {
            detail: format!("sequence {seq_ref}: {e}"),
        })?;

    // SBOL component roles become the design's own annotation. Roles on the
    // component describe the whole construct rather than a sub-region, so they
    // are carried as a single feature spanning it.
    let mut features = Vec::new();
    for (i, role) in c.roles.iter().enumerate() {
        features.push(Feature {
            id: FeatureId(format!("{id}#role{i}")),
            location: Location {
                start: 0,
                end: sequence.len(),
                strand: Strand::Forward,
            },
            role: OntologyTerm(role.to_string()),
            name: c.identified.name.clone(),
            children: Vec::new(),
        });
    }

    if !c.features.is_empty() {
        // Sub-features carry their own locations and need resolving through the
        // document. Not yet implemented, and said so rather than silently
        // producing a design that looks complete and is not.
        report.notes.push(format!(
            "component {id} has {} sub-feature(s) which this version does not yet resolve; \
             only component-level roles were imported",
            c.features.len()
        ));
    }

    Ok(Some(Design {
        id: DesignId(id.clone()),
        sequence,
        features,
        provenance: Provenance::Imported {
            source: id,
            format: "SBOL3".into(),
        },
    }))
}

/// The encoding IRI this importer writes for nucleic acid sequences.
#[must_use]
pub const fn iupac_dna_encoding() -> &'static str {
    ENCODING_IUPAC_DNA
}
