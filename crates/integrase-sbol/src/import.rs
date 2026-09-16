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

/// SBOL encoding IRI for IUPAC nucleic acid sequences.
const ENCODING_IUPAC_DNA: &str = "https://identifiers.org/edam:format_1207";

/// EDAM format for pure nucleotide sequence.
///
/// Matched as a substring because the same term appears as
/// `https://identifiers.org/edam:format_1207` and as
/// `http://edamontology.org/format_1207` in files that are otherwise identical.
const EDAM_NUCLEIC_ACID: &str = "1207";

/// EDAM format for pure protein sequence.
const EDAM_PROTEIN: &str = "1208";

/// EDAM format for SMILES, which describes a small molecule rather than a
/// polymer sequence.
const EDAM_SMILES: &str = "1196";

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

    // Index the typed objects once. `Document::resolve` is a linear scan, and
    // resolving every feature and every location through it would be quadratic
    // in document size.
    let index: std::collections::BTreeMap<&sbol3::Resource, &sbol3::SbolObject> = doc
        .typed_objects()
        .iter()
        .map(|o| (o.identity(), o))
        .collect();

    let mut sequences: std::collections::BTreeMap<String, &sbol3::Sequence> =
        std::collections::BTreeMap::new();
    for obj in doc.typed_objects() {
        if let sbol3::SbolObject::Sequence(s) = obj {
            sequences.insert(s.identity.to_string(), s);
        }
    }

    let mut designs = Vec::new();
    for obj in doc.typed_objects() {
        // Matching the typed variant rather than projecting every object. The
        // `try_from_object` constructors are projections, not type tests: each
        // returns `Some` for any object, reading whatever properties happen to
        // be present. A `sbol:Range` legitimately carries `sbol:hasSequence`,
        // pointing at the sequence it indexes into, so projecting one as a
        // Component turned a coordinate pair into a plausible-looking design.
        let sbol3::SbolObject::Component(c) = obj else {
            continue;
        };
        match component_to_design(c, &index, &sequences, &mut report) {
            Ok(Some(d)) => designs.push(d),
            Ok(None) => {}
            Err(e) => return Err(e),
        }
    }

    // A document that yields nothing has to say what it did contain. Filtering
    // by class removed the incidental notes that used to explain an empty
    // result, and "no designs, no explanation" reads as a broken importer.
    if designs.is_empty() {
        let mut classes: Vec<&str> = doc
            .typed_objects()
            .iter()
            .map(|o| o.class().local_name())
            .collect();
        classes.sort_unstable();
        classes.dedup();
        report.notes.push(if classes.is_empty() {
            "this document contains no SBOL objects".to_owned()
        } else {
            format!(
                "this document declares no sbol:Component with a sequence; it contains: {}",
                classes.join(", ")
            )
        });
    }

    Ok(ImportResult { designs, report })
}

/// Translates one component, if it carries sequence we can read.
fn component_to_design(
    c: &sbol3::Component,
    index: &ObjectIndex<'_>,
    sequences: &std::collections::BTreeMap<String, &sbol3::Sequence>,
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
        Some(e) if e.contains(EDAM_NUCLEIC_ACID) || e.contains("DNA") || e.contains("dna") => {
            Alphabet::DnaIupac
        }
        // A protein or small-molecule sequence is skipped, not coerced.
        //
        // The single-letter amino acid alphabet overlaps the IUPAC nucleotide
        // ambiguity codes almost completely, so a protein sequence passes
        // nucleotide validation and becomes a "DNA design" holding no DNA.
        // Every downstream number computed from it would be meaningless and
        // nothing about it would look wrong.
        Some(other)
            if other.contains(EDAM_PROTEIN)
                || other.contains(EDAM_SMILES)
                || other.contains("protein") =>
        {
            report.notes.push(format!(
                "sequence {seq_ref} declares encoding {other}, which is not a nucleic acid; \
                 component {id} was skipped rather than reinterpreted as DNA"
            ));
            return Ok(None);
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

    features.extend(resolve_features(c, index, sequence.len(), report));

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

/// Every typed object in a document, keyed by identity.
///
/// Built once per import. `Document::resolve` scans linearly, and every feature
/// and every location is a reference that has to be followed.
type ObjectIndex<'d> = std::collections::BTreeMap<&'d sbol3::Resource, &'d sbol3::SbolObject>;

/// Resolves a component's sub-features into located annotations.
///
/// Each SBOL feature points at one or more location objects, and each location
/// is a separate object in the document that has to be followed. A feature with
/// several locations becomes several `Feature` values, because
/// [`integrase_core::Location`] describes a single contiguous span and
/// flattening a multi-part feature into its bounding box would claim the gaps
/// between the parts belong to it.
fn resolve_features(
    c: &sbol3::Component,
    index: &ObjectIndex<'_>,
    seq_len: usize,
    report: &mut ConversionReport,
) -> Vec<Feature> {
    let mut out = Vec::new();

    for feature_ref in &c.features {
        let Some(obj) = index.get(feature_ref) else {
            report.notes.push(format!(
                "feature {feature_ref} is referenced but not present in this document"
            ));
            continue;
        };

        let (locations, own_roles, (authored_name, structural), definition) = match obj {
            sbol3::SbolObject::SubComponent(sc) => (
                &sc.locations,
                &sc.feature.roles,
                (&sc.identified.name, &sc.identified.display_id),
                // A sub-component usually carries no role of its own: the role
                // belongs to the part it instantiates. Following `instanceOf`
                // is what turns an anonymous span into "this is a promoter".
                sc.instance_of
                    .as_ref()
                    .and_then(|r| index.get(r))
                    .and_then(|o| match o {
                        sbol3::SbolObject::Component(rc) => Some(rc),
                        _ => None,
                    }),
            ),
            sbol3::SbolObject::SequenceFeature(sf) => (
                &sf.locations,
                &sf.feature.roles,
                (&sf.identified.name, &sf.identified.display_id),
                None,
            ),
            sbol3::SbolObject::LocalSubComponent(lc) => (
                &lc.locations,
                &lc.feature.roles,
                (&lc.identified.name, &lc.identified.display_id),
                None,
            ),
            // `ComponentReference` points into another component's instance
            // tree and `ExternallyDefined` names something outside SBOL
            // entirely. Neither resolves to a span on this sequence without
            // machinery that does not exist yet, so both are reported rather
            // than drawn somewhere plausible.
            other => {
                report.notes.push(format!(
                    "feature {feature_ref} is a {}, which this version does not resolve to a \
                     location",
                    other.class().local_name()
                ));
                continue;
            }
        };

        let roles: Vec<String> = if own_roles.is_empty() {
            definition
                .map(|rc| rc.roles.iter().map(ToString::to_string).collect())
                .unwrap_or_default()
        } else {
            own_roles.iter().map(ToString::to_string).collect()
        };
        if roles.len() > 1 {
            report.notes.push(format!(
                "feature {feature_ref} declares {} roles; the first was used",
                roles.len()
            ));
        }
        // A feature with no role still gets carried, with an empty role IRI,
        // which the renderer shows as an unrecognised glyph. That is the
        // truthful outcome: the span is real, and what occupies it is not
        // stated in the file.
        let role = roles.into_iter().next().unwrap_or_default();
        // Naming order matters more than it looks. A sub-component's own
        // `displayId` is a structural artefact of the serialisation, typically
        // "SubComponent3", while the part it instantiates is called "BBa_R0040",
        // which is what a reader recognises. An explicit `name` on the
        // sub-component is authored text and still wins over both.
        let name = authored_name
            .clone()
            .or_else(|| definition.and_then(|rc| label(&rc.identified)))
            .or_else(|| structural.clone());

        if locations.is_empty() {
            report.notes.push(format!(
                "feature {feature_ref} has no location and was skipped"
            ));
            continue;
        }

        for (i, loc_ref) in locations.iter().enumerate() {
            let Some(location) = resolve_location(loc_ref, index, seq_len, report) else {
                continue;
            };
            let suffix = if locations.len() > 1 {
                format!("#loc{i}")
            } else {
                String::new()
            };
            out.push(Feature {
                id: FeatureId(format!("{feature_ref}{suffix}")),
                location,
                role: OntologyTerm(role.clone()),
                name: name.clone(),
                children: Vec::new(),
            });
        }
    }

    // Sorted by span. Import order follows RDF subject order, which is an
    // artefact of the serialisation, and two files describing the same
    // construct should produce the same design.
    out.sort_by(|a, b| {
        (a.location.start, a.location.end, &a.id.0).cmp(&(
            b.location.start,
            b.location.end,
            &b.id.0,
        ))
    });
    out
}

/// Follows one location reference to a span on the parent sequence.
///
/// SBOL ranges are 1-based and inclusive at both ends. `integrase-core` uses
/// 0-based starts and exclusive ends, so a `Range` of 1 to 80 becomes 0 to 80.
/// Getting this off by one shifts every annotation in the file, which is the
/// kind of error that looks fine right up until someone orders the DNA.
fn resolve_location(
    loc_ref: &sbol3::Resource,
    index: &ObjectIndex<'_>,
    seq_len: usize,
    report: &mut ConversionReport,
) -> Option<Location> {
    let Some(obj) = index.get(loc_ref) else {
        report.notes.push(format!(
            "location {loc_ref} is referenced but not present in this document"
        ));
        return None;
    };

    match obj {
        sbol3::SbolObject::Range(r) => {
            let strand = strand_from(r.location.orientation.as_ref(), loc_ref, report);
            let (Some(start), Some(end)) = (r.start, r.end) else {
                report.notes.push(format!(
                    "range {loc_ref} is missing a start or an end and was skipped"
                ));
                return None;
            };
            if start < 1 || end < start {
                report.notes.push(format!(
                    "range {loc_ref} declares start {start} and end {end}, which is not a valid \
                     1-based inclusive span; skipped"
                ));
                return None;
            }
            let start0 = usize::try_from(start - 1).ok()?;
            let end0 = usize::try_from(end).ok()?;
            if end0 > seq_len {
                report.notes.push(format!(
                    "range {loc_ref} ends at {end0} but the sequence is {seq_len} long; imported \
                     as written rather than clamped, so the design will fail bounds validation"
                ));
            }
            Some(Location {
                start: start0,
                end: end0,
                strand,
            })
        }
        // A cut is a zero-width position between two bases, not a span. It is
        // carried as an empty range there rather than widened to cover a base,
        // because widening would invent an annotation the file does not make.
        sbol3::SbolObject::Cut(cut) => {
            let strand = strand_from(cut.location.orientation.as_ref(), loc_ref, report);
            let Some(at) = cut.at else {
                report
                    .notes
                    .push(format!("cut {loc_ref} has no position and was skipped"));
                return None;
            };
            let at0 = usize::try_from(at).ok()?;
            Some(Location {
                start: at0,
                end: at0,
                strand,
            })
        }
        sbol3::SbolObject::EntireSequence(es) => {
            let strand = strand_from(es.location.orientation.as_ref(), loc_ref, report);
            Some(Location {
                start: 0,
                end: seq_len,
                strand,
            })
        }
        other => {
            report.notes.push(format!(
                "location {loc_ref} is a {}, which this version does not understand",
                other.class().local_name()
            ));
            None
        }
    }
}

/// Reads an SBOL orientation IRI as a strand.
///
/// An absent orientation defaults to inline, which is what SBOL 3 specifies. An
/// orientation this version does not recognise is reported rather than defaulted
/// in silence, because quietly treating a reverse-complement feature as forward
/// puts it on the wrong strand in every view downstream.
fn strand_from(
    orientation: Option<&sbol3::Iri>,
    context: &sbol3::Resource,
    report: &mut ConversionReport,
) -> Strand {
    let Some(o) = orientation else {
        return Strand::Forward;
    };
    let s = o.to_string();
    if s.ends_with(SO_INLINE_ACCESSION) {
        Strand::Forward
    } else if s.ends_with(SO_REVERSE_COMPLEMENT_ACCESSION) {
        Strand::Reverse
    } else {
        report.notes.push(format!(
            "{context} declares orientation {s}, which is neither inline nor reverseComplement; \
             imported as inline"
        ));
        Strand::Forward
    }
}

/// Sequence Ontology accession for inline orientation.
///
/// Matched as a suffix rather than a whole IRI because the same term is written
/// `https://identifiers.org/SO:0001030` and
/// `http://purl.obolibrary.org/obo/SO_0001030` in files that are otherwise
/// identical.
const SO_INLINE_ACCESSION: &str = "0001030";

/// Sequence Ontology accession for reverse-complement orientation.
const SO_REVERSE_COMPLEMENT_ACCESSION: &str = "0001031";

/// A human-readable label, preferring `name` over `displayId`.
fn label(d: &sbol3::IdentifiedData) -> Option<String> {
    d.name.clone().or_else(|| d.display_id.clone())
}

/// The encoding IRI this importer writes for nucleic acid sequences.
#[must_use]
pub const fn iupac_dna_encoding() -> &'static str {
    ENCODING_IUPAC_DNA
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal document with one component and two located sub-features.
    ///
    /// Written by hand rather than taken from the corpus so the coordinates are
    /// known exactly. The off-by-one between SBOL's 1-based inclusive ranges and
    /// our 0-based exclusive ones is the kind of error a corpus test cannot
    /// catch, because every fixture would be shifted consistently and still look
    /// self-consistent.
    const TWO_PARTS: &str = r#"
@prefix sbol: <http://sbols.org/v3#> .
@prefix SO: <https://identifiers.org/SO:> .
@prefix ex: <https://example.org/> .

ex:c1 a sbol:Component ;
    sbol:displayId "c1" ;
    sbol:hasNamespace <https://example.org> ;
    sbol:type SO:0000110 ;
    sbol:hasSequence ex:s1 ;
    sbol:hasFeature ex:c1_f1, ex:c1_f2 .

ex:s1 a sbol:Sequence ;
    sbol:displayId "s1" ;
    sbol:hasNamespace <https://example.org> ;
    sbol:elements "AAAACCCCGGGGTTTT" ;
    sbol:encoding <https://identifiers.org/edam:format_1207> .

ex:c1_f1 a sbol:SubComponent ;
    sbol:displayId "SubComponent1" ;
    sbol:instanceOf ex:part_a ;
    sbol:hasLocation ex:c1_f1_r1 .

ex:c1_f1_r1 a sbol:Range ;
    sbol:displayId "Range1" ;
    sbol:start "1" ;
    sbol:end "4" ;
    sbol:orientation SO:0001030 .

ex:c1_f2 a sbol:SubComponent ;
    sbol:displayId "SubComponent2" ;
    sbol:instanceOf ex:part_b ;
    sbol:hasLocation ex:c1_f2_r1 .

ex:c1_f2_r1 a sbol:Range ;
    sbol:displayId "Range1" ;
    sbol:start "9" ;
    sbol:end "16" ;
    sbol:orientation SO:0001031 .

ex:part_a a sbol:Component ;
    sbol:displayId "part_a" ;
    sbol:name "Promoter A" ;
    sbol:hasNamespace <https://example.org> ;
    sbol:type SO:0000110 ;
    sbol:role SO:0000167 .

ex:part_b a sbol:Component ;
    sbol:displayId "part_b" ;
    sbol:hasNamespace <https://example.org> ;
    sbol:type SO:0000110 ;
    sbol:role SO:0000316 .
"#;

    fn only_design(turtle: &str) -> Design {
        let r = from_turtle(turtle).expect("fixture parses");
        let mut designs: Vec<Design> = r
            .designs
            .into_iter()
            .filter(|d| !d.sequence.is_empty())
            .collect();
        assert_eq!(
            designs.len(),
            1,
            "expected exactly one design with sequence"
        );
        designs.remove(0)
    }

    /// SBOL ranges are 1-based and inclusive at both ends.
    ///
    /// A range of 1 to 4 covers the first four bases, which is `0..4` here. A
    /// range of 9 to 16 covers the last eight of a sixteen-base sequence, which
    /// is `8..16`. Getting either wrong shifts every annotation in every file.
    #[test]
    fn ranges_convert_from_one_based_inclusive() {
        let d = only_design(TWO_PARTS);
        let spans: Vec<(usize, usize)> = d
            .features
            .iter()
            .map(|f| (f.location.start, f.location.end))
            .collect();
        assert!(spans.contains(&(0, 4)), "got {spans:?}");
        assert!(spans.contains(&(8, 16)), "got {spans:?}");
    }

    #[test]
    fn orientation_becomes_strand() {
        let d = only_design(TWO_PARTS);
        let forward = d
            .features
            .iter()
            .find(|f| f.location.start == 0)
            .expect("first feature");
        let reverse = d
            .features
            .iter()
            .find(|f| f.location.start == 8)
            .expect("second feature");
        assert_eq!(forward.location.strand, Strand::Forward);
        assert_eq!(reverse.location.strand, Strand::Reverse);
    }

    /// A sub-component carries no role of its own; the role belongs to the part
    /// it instantiates. Without following `instanceOf` every feature would
    /// render as an unrecognised glyph.
    #[test]
    fn roles_come_from_the_instantiated_part() {
        let d = only_design(TWO_PARTS);
        let roles: Vec<&str> = d.features.iter().map(|f| f.role.0.as_str()).collect();
        assert!(roles.iter().any(|r| r.ends_with("SO:0000167")), "{roles:?}");
        assert!(roles.iter().any(|r| r.ends_with("SO:0000316")), "{roles:?}");
    }

    /// The instantiated part's name beats the sub-component's `displayId`.
    ///
    /// "SubComponent1" is an artefact of the serialisation. "Promoter A" is what
    /// the author wrote. When the part has no name either, its `displayId`
    /// ("part_b") is still more useful than the structural one.
    #[test]
    fn names_prefer_the_instantiated_part() {
        let d = only_design(TWO_PARTS);
        let names: Vec<&str> = d
            .features
            .iter()
            .filter_map(|f| f.name.as_deref())
            .collect();
        assert!(names.contains(&"Promoter A"), "{names:?}");
        assert!(names.contains(&"part_b"), "{names:?}");
        assert!(
            !names.iter().any(|n| n.starts_with("SubComponent")),
            "structural identifier leaked into a label: {names:?}"
        );
    }

    /// A `sbol:Range` carries `sbol:hasSequence`, pointing at the sequence it
    /// indexes into. The typed constructors are projections rather than type
    /// tests, so before the class check a range projected cleanly into a
    /// Component and a coordinate pair became a design.
    #[test]
    fn a_range_is_not_a_design() {
        let r = from_turtle(TWO_PARTS).expect("fixture parses");
        for d in &r.designs {
            assert!(
                !d.id.0.contains("_r1"),
                "a location object was imported as a design: {}",
                d.id.0
            );
        }
        // Only `c1`. The two instantiated parts carry roles but no sequence of
        // their own, so they are skipped, and the two ranges are locations
        // rather than constructs.
        assert_eq!(
            r.designs.len(),
            1,
            "{:?}",
            r.designs.iter().map(|d| &d.id.0).collect::<Vec<_>>()
        );
        assert_eq!(
            r.report
                .notes
                .iter()
                .filter(|n| n.contains("has no sequence"))
                .count(),
            2,
            "both sequenceless parts must be accounted for: {:?}",
            r.report.notes
        );
    }

    /// Feature order must not depend on RDF subject order.
    #[test]
    fn features_are_ordered_by_span() {
        let d = only_design(TWO_PARTS);
        let starts: Vec<usize> = d.features.iter().map(|f| f.location.start).collect();
        let mut sorted = starts.clone();
        sorted.sort_unstable();
        assert_eq!(starts, sorted, "features are not in sequence order");
    }

    /// The single-letter amino acid alphabet overlaps the IUPAC nucleotide
    /// ambiguity codes, so a protein sequence passes nucleotide validation. It
    /// has to be rejected on its declared encoding, before validation, or it
    /// becomes a DNA design containing no DNA.
    #[test]
    fn a_protein_sequence_is_skipped_rather_than_read_as_dna() {
        let turtle = r#"
@prefix sbol: <http://sbols.org/v3#> .
@prefix SO: <https://identifiers.org/SO:> .
@prefix ex: <https://example.org/> .

ex:p a sbol:Component ;
    sbol:displayId "p" ;
    sbol:hasNamespace <https://example.org> ;
    sbol:type SO:0000104 ;
    sbol:hasSequence ex:ps .

ex:ps a sbol:Sequence ;
    sbol:displayId "ps" ;
    sbol:hasNamespace <https://example.org> ;
    sbol:elements "MKTAYIAKQR" ;
    sbol:encoding <https://identifiers.org/edam:format_1208> .
"#;
        let r = from_turtle(turtle).expect("fixture parses");
        assert!(
            r.designs.is_empty(),
            "a protein sequence became a design: {:?}",
            r.designs.iter().map(|d| &d.id.0).collect::<Vec<_>>()
        );
        assert!(
            r.report
                .notes
                .iter()
                .any(|n| n.contains("not a nucleic acid")),
            "the skip must be explained: {:?}",
            r.report.notes
        );
    }

    /// An out-of-range end is reported rather than clamped.
    ///
    /// Clamping would produce a design that passes bounds validation while
    /// silently disagreeing with the file it came from.
    #[test]
    fn an_out_of_bounds_range_is_reported() {
        let turtle = TWO_PARTS.replace(r#"sbol:end "16""#, r#"sbol:end "900""#);
        let r = from_turtle(&turtle).expect("fixture parses");
        assert!(
            r.report
                .notes
                .iter()
                .any(|n| n.contains("but the sequence is")),
            "{:?}",
            r.report.notes
        );
    }

    /// A reversed or zero start is refused outright, since there is no honest
    /// interpretation of it.
    #[test]
    fn an_invalid_range_is_skipped_with_a_reason() {
        let turtle = TWO_PARTS.replace(r#"sbol:start "9""#, r#"sbol:start "40""#);
        let r = from_turtle(&turtle).expect("fixture parses");
        assert!(
            r.report
                .notes
                .iter()
                .any(|n| n.contains("1-based inclusive span")),
            "{:?}",
            r.report.notes
        );
    }

    /// An unrecognised orientation defaults to inline and says so.
    #[test]
    fn an_unknown_orientation_is_reported() {
        let turtle = TWO_PARTS.replace("SO:0001031", "SO:9999999");
        let r = from_turtle(&turtle).expect("fixture parses");
        assert!(
            r.report
                .notes
                .iter()
                .any(|n| n.contains("neither inline nor")),
            "{:?}",
            r.report.notes
        );
    }
}
