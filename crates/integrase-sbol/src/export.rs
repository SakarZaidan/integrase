//! Integrase to SBOL 3.
//!
//! # Two export classes, and the distinction is the flow
//!
//! Subplan 21 amended subplan 06 on this, and the amendment is load-bearing:
//!
//! - **Interchange** ([`write_sbol`]) is **not gated**. A design that came from
//!   the ecosystem returns to it, and we are not the synthesis boundary there.
//!   Gating a user's ability to save their own working file would make the tool
//!   unusable while stopping nobody.
//! - **Synthesis-bound** ([`export_for_synthesis`]) requires
//!   [`integrase_screen::Cleared`], which only the screening gateway can
//!   construct. Bypassing it does not compile.
//!
//! The same bytes can be either. A FASTA file a user saves to keep working is
//! interchange; the same FASTA produced by an ordering flow is synthesis-bound.
//! **That is why the gate lives on the function rather than on the serialiser.**

use crate::SbolError;
use integrase_core::{Design, Feature};
use integrase_screen::Cleared;
use sbol3::prelude::*;

// Use the constant sbol-rs already defines rather than restating the IRI. A
// hand-copied ontology IRI is a typo waiting to happen, and it would fail
// silently as an unrecognised term rather than as an error.
use sbol3::constants::SBO_DNA;

/// Formats a design can be written toward a synthesis vendor in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum VendorFormat {
    /// Plain FASTA, which most synthesis vendors accept.
    Fasta,
}

/// Writes a design as SBOL 3 Turtle.
///
/// **Interchange. Not gated, deliberately.** See the module documentation.
///
/// # Errors
///
/// [`SbolError`] if the design cannot be represented, for example because its
/// identifier is not a usable SBOL display id.
pub fn write_sbol(design: &Design) -> Result<String, SbolError> {
    let namespace = "https://integrase.dev/design";
    let display_id = sanitise_display_id(&design.id.0);

    let sequence = sbol3::Sequence::builder(namespace, format!("{display_id}_sequence"))
        .map_err(|e| SbolError::Read {
            detail: e.to_string(),
        })?
        .elements(String::from_utf8_lossy(design.sequence.as_bytes()).to_string())
        .build()
        .map_err(|e| SbolError::Read {
            detail: e.to_string(),
        })?;

    let mut builder = sbol3::Component::builder(namespace, &display_id)
        .map_err(|e| SbolError::Read {
            detail: e.to_string(),
        })?
        .types([SBO_DNA])
        .sequences([sequence.identity.clone()]);

    // Component-level roles are carried across. Sub-feature locations are not
    // round-tripped yet, matching the import side.
    //
    // An unparseable role IRI is an error rather than a silent skip: dropping a
    // role would produce a document that looks complete and has quietly lost
    // annotation, which is the failure mode the import side reports on.
    for role in roles_of(&design.features) {
        let iri = Iri::new(role.clone()).map_err(|_| SbolError::UnresolvedTerm { iri: role })?;
        builder = builder.add_component_role(iri);
    }

    let component = builder.build().map_err(|e| SbolError::Read {
        detail: e.to_string(),
    })?;

    let doc = sbol3::Document::from_objects(vec![
        SbolObject::Component(component),
        SbolObject::Sequence(sequence),
    ])
    .map_err(|e| SbolError::Read {
        detail: e.to_string(),
    })?;

    doc.write_turtle().map_err(|e| SbolError::Read {
        detail: e.to_string(),
    })
}

/// Writes a design toward physical synthesis.
///
/// **This is the gate.** The signature takes [`Cleared`], which is constructible
/// only inside `integrase-screen`. There is no way to call this with an
/// unscreened design, and a `trybuild` test asserts that the attempt does not
/// compile.
///
/// # Errors
///
/// [`SbolError`] if the design cannot be serialised.
pub fn export_for_synthesis(
    cleared: &Cleared<Design>,
    format: VendorFormat,
) -> Result<String, SbolError> {
    let design = cleared.get();
    match format {
        VendorFormat::Fasta => {
            let seq = String::from_utf8_lossy(design.sequence.as_bytes());
            // The screening verdict travels with the export. A vendor receiving
            // this can see which backend and policy version cleared it, which is
            // the point of recording them on the verdict at all.
            let v = cleared.verdict();
            Ok(format!(
                ">{} screened_by={} policy={}\n{}\n",
                design.id.0, v.backend.0, v.policy_version.0, seq
            ))
        }
    }
}

/// Distinct role IRIs across a design's features, in first-seen order.
fn roles_of(features: &[Feature]) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for f in features {
        if seen.insert(f.role.0.clone()) {
            out.push(f.role.0.clone());
        }
    }
    out
}

/// Makes an arbitrary identifier usable as an SBOL display id.
///
/// SBOL display ids must be alphanumeric or underscore and may not lead with a
/// digit. Imported identifiers are frequently full IRIs, so the last path
/// segment is taken and anything else replaced.
fn sanitise_display_id(raw: &str) -> String {
    let tail = raw.rsplit('/').next().unwrap_or(raw);
    let mut s: String = tail
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    if s.is_empty() {
        s.push_str("design");
    }
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        s.insert(0, 'd');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use integrase_core::{
        Alphabet, DesignId, FeatureId, Location, OntologyTerm, Provenance, Sequence, Strand,
    };

    fn design(id: &str, seq: &str, roles: &[OntologyTerm]) -> Design {
        Design {
            id: DesignId(id.into()),
            sequence: Sequence::new(seq.as_bytes().to_vec(), Alphabet::DnaIupac).unwrap(),
            features: roles
                .iter()
                .enumerate()
                .map(|(i, r)| Feature {
                    id: FeatureId(format!("f{i}")),
                    location: Location {
                        start: 0,
                        end: seq.len(),
                        strand: Strand::Forward,
                    },
                    role: r.clone(),
                    name: None,
                    children: Vec::new(),
                })
                .collect(),
            provenance: Provenance::Authored,
        }
    }

    #[test]
    fn a_design_writes_valid_sbol() {
        let d = design("plasmid1", "ACGTACGTACGT", &[OntologyTerm::promoter()]);
        let turtle = write_sbol(&d).expect("should write");
        assert!(
            turtle.contains("Component"),
            "output must declare a Component"
        );
        assert!(
            turtle.contains("ACGTACGTACGT"),
            "the sequence must be present"
        );
    }

    /// **The round trip.** Export then import must preserve the sequence.
    ///
    /// Compared semantically, not byte-for-byte: the identifier is rewritten to
    /// a valid SBOL display id and namespace, so a textual comparison would fail
    /// on correct behaviour (R-2.4).
    #[test]
    fn a_design_survives_a_round_trip_through_sbol() {
        let original = design(
            "plasmid1",
            "ACGTACGTACGTAAGGCC",
            &[OntologyTerm::promoter()],
        );
        let turtle = write_sbol(&original).expect("write");
        let back = crate::from_turtle(&turtle).expect("read back");

        assert_eq!(back.designs.len(), 1, "one component in, one design out");
        assert_eq!(
            back.designs[0].sequence.as_bytes(),
            original.sequence.as_bytes(),
            "the sequence must survive the round trip exactly"
        );
    }

    #[test]
    fn roles_survive_the_round_trip() {
        let original = design("p", "ACGTACGTACGT", &[OntologyTerm::promoter()]);
        let turtle = write_sbol(&original).expect("write");
        let back = crate::from_turtle(&turtle).expect("read");
        let roles: Vec<String> = back.designs[0]
            .features
            .iter()
            .map(|f| f.role.0.clone())
            .collect();
        assert!(
            roles.iter().any(|r| r == &OntologyTerm::promoter().0),
            "the promoter role must survive; got {roles:?}"
        );
    }

    #[test]
    fn an_iri_that_is_not_an_iri_is_refused_rather_than_dropped() {
        let d = design(
            "p",
            "ACGTACGTACGT",
            &[OntologyTerm("not a valid iri at all".into())],
        );
        match write_sbol(&d) {
            Err(SbolError::UnresolvedTerm { iri }) => assert!(iri.contains("not a valid")),
            Ok(_) => panic!("a bad role must not be silently dropped from the output"),
            Err(other) => panic!("wrong error: {other:?}"),
        }
    }

    // --- Display id sanitisation -----------------------------------------

    #[test]
    fn an_iri_identifier_becomes_a_usable_display_id() {
        assert_eq!(
            sanitise_display_id("https://example.org/lab/my_part"),
            "my_part"
        );
    }

    #[test]
    fn a_leading_digit_is_prefixed_because_sbol_forbids_it() {
        assert_eq!(sanitise_display_id("123abc"), "d123abc");
    }

    #[test]
    fn an_empty_identifier_gets_a_fallback_rather_than_producing_invalid_sbol() {
        assert_eq!(sanitise_display_id(""), "design");
    }

    #[test]
    fn punctuation_is_replaced_not_dropped() {
        // Dropping would collide two different identifiers into one.
        assert_eq!(sanitise_display_id("a-b.c"), "a_b_c");
    }

    // --- The gate ---------------------------------------------------------

    // The signature guarantee is proved by `tests/gate_compile_fail.rs`, which
    // asserts that calling `export_for_synthesis` with a bare `Design` does not
    // compile, and pins the exact error.
    //
    // An earlier version of this module also grepped its own source text for the
    // signature. That was a weaker duplicate of the trybuild test and brittle in
    // a way that showed up on exactly one platform: Windows checks out CRLF, so
    // matching a literal "(\n    cleared:" failed there and nowhere else.
    // Removed rather than patched, because the coverage was already there.
}
