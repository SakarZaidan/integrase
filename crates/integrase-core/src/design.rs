//! Designs: located, annotated constructs over a sequence.

use crate::{provenance::Provenance, seq::Sequence};

/// Stable identifier for a design.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DesignId(pub String);

/// Stable identifier for a feature within a design.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct FeatureId(pub String);

pub use crate::seq::Strand;

/// A term from a controlled vocabulary, held as an IRI.
///
/// Roles are IRIs rather than an enum, deliberately (subplan 06). The Sequence
/// Ontology has thousands of terms, gains more, and is not ours to enumerate. An
/// enum would force a breaking change every time biology adds a category, plus
/// an `Other(String)` escape hatch that becomes the most-used variant. IRIs also
/// round-trip through SBOL without translation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct OntologyTerm(pub String);

impl OntologyTerm {
    /// Sequence Ontology term for a promoter.
    #[must_use]
    pub fn promoter() -> Self {
        Self("https://identifiers.org/SO:0000167".into())
    }

    /// Sequence Ontology term for a ribosome entry site.
    #[must_use]
    pub fn rbs() -> Self {
        Self("https://identifiers.org/SO:0000139".into())
    }

    /// Sequence Ontology term for a CDS.
    #[must_use]
    pub fn cds() -> Self {
        Self("https://identifiers.org/SO:0000316".into())
    }

    /// Sequence Ontology term for a terminator.
    #[must_use]
    pub fn terminator() -> Self {
        Self("https://identifiers.org/SO:0000141".into())
    }
}

/// A half-open, strand-aware span. 0-based, `end` exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Location {
    /// Inclusive start, 0-based.
    pub start: usize,
    /// Exclusive end.
    pub end: usize,
    /// Which strand the feature is on.
    pub strand: Strand,
}

impl Location {
    /// Length of the span in residues.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Returns true if the span covers no residues.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Returns true if the two spans share at least one residue.
    ///
    /// Strand-agnostic: overlap on opposite strands is still overlap, and it is
    /// biologically real (subplan 11, edge case E6).
    #[must_use]
    pub const fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}

/// A located, role-annotated region of a design.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Feature {
    /// Identifier, unique within the design.
    pub id: FeatureId,
    /// Where it sits.
    pub location: Location,
    /// What it is, as an ontology IRI.
    pub role: OntologyTerm,
    /// Human-readable name, if the source supplied one.
    pub name: Option<String>,
    /// Nested sub-features, by identifier.
    pub children: Vec<FeatureId>,
}

/// A construct: a sequence plus the features annotated on it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Design {
    /// Identifier for this design.
    pub id: DesignId,
    /// The underlying residues.
    pub sequence: Sequence,
    /// Annotated regions. Order is preserved as imported; overlap is permitted.
    pub features: Vec<Feature>,
    /// Where this design came from.
    pub provenance: Provenance,
}

impl Design {
    /// Returns every feature whose role matches `role`.
    pub fn features_with_role<'a>(
        &'a self,
        role: &'a OntologyTerm,
    ) -> impl Iterator<Item = &'a Feature> {
        self.features.iter().filter(move |f| &f.role == role)
    }

    /// Returns features whose location falls outside the sequence.
    ///
    /// Non-empty output means the design is internally inconsistent, which real
    /// imported files sometimes are. Reporting rather than silently clamping is
    /// deliberate.
    #[must_use]
    pub fn out_of_bounds_features(&self) -> Vec<&Feature> {
        let len = self.sequence.len();
        self.features
            .iter()
            .filter(|f| f.location.end > len)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seq::Alphabet;

    fn design_with(features: Vec<Feature>, len: usize) -> Design {
        Design {
            id: DesignId("d1".into()),
            sequence: Sequence::new(vec![b'A'; len], Alphabet::Dna).unwrap(),
            features,
            provenance: Provenance::Authored,
        }
    }

    fn feat(id: &str, start: usize, end: usize, role: OntologyTerm) -> Feature {
        Feature {
            id: FeatureId(id.into()),
            location: Location {
                start,
                end,
                strand: Strand::Forward,
            },
            role,
            name: None,
            children: vec![],
        }
    }

    #[test]
    fn overlap_is_detected_on_shared_residues_only() {
        let a = Location {
            start: 0,
            end: 10,
            strand: Strand::Forward,
        };
        let b = Location {
            start: 10,
            end: 20,
            strand: Strand::Forward,
        };
        let c = Location {
            start: 9,
            end: 20,
            strand: Strand::Forward,
        };
        assert!(!a.overlaps(&b), "half-open spans that touch do not overlap");
        assert!(a.overlaps(&c), "spans sharing residue 9 do overlap");
    }

    #[test]
    fn overlap_ignores_strand_because_overlap_is_physical() {
        let fwd = Location {
            start: 0,
            end: 10,
            strand: Strand::Forward,
        };
        let rev = Location {
            start: 5,
            end: 15,
            strand: Strand::Reverse,
        };
        assert!(fwd.overlaps(&rev));
    }

    #[test]
    fn out_of_bounds_features_are_reported_not_clamped() {
        let d = design_with(
            vec![
                feat("f1", 0, 5, OntologyTerm::cds()),
                feat("f2", 8, 20, OntologyTerm::cds()),
            ],
            10,
        );
        let bad = d.out_of_bounds_features();
        assert_eq!(bad.len(), 1);
        assert_eq!(bad[0].id, FeatureId("f2".into()));
    }

    #[test]
    fn role_filter_matches_by_iri() {
        let d = design_with(
            vec![
                feat("p", 0, 3, OntologyTerm::promoter()),
                feat("c", 3, 9, OntologyTerm::cds()),
            ],
            10,
        );
        let promoter = OntologyTerm::promoter();
        let promoters: Vec<_> = d.features_with_role(&promoter).collect();
        assert_eq!(promoters.len(), 1);
        assert_eq!(promoters[0].id, FeatureId("p".into()));
    }
}
