//! SBOL Visual glyph selection and geometry.
//!
//! # Selection is an ontology lookup, not a match statement
//!
//! A feature's role is a Sequence Ontology IRI, and the glyph follows from it
//! through a table. A `match` over role names would drift from the standard the
//! first time the SO adds a term, and the drift would be silent: an unrecognised
//! role would fall through to a default and the diagram would quietly misrepresent
//! the design.
//!
//! Unrecognised roles render as an explicit unknown glyph rather than as a
//! plausible-looking guess, for the same reason a prediction outside a model's
//! domain is flagged rather than extrapolated.

/// The shapes SBOL Visual defines for the parts this renderer draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Glyph {
    /// Bent arrow. Transcription start.
    Promoter,
    /// Dome. Ribosome entry site.
    RibosomeEntrySite,
    /// Arrow-shaped box. Protein-coding sequence.
    Cds,
    /// T-bar. Transcription stop.
    Terminator,
    /// Circle. Origin of replication.
    OriginOfReplication,
    /// Plain box. A recognised feature with no specific glyph.
    Generic,
    /// Dashed box. **The role was not recognised.**
    ///
    /// Deliberately distinct from [`Glyph::Generic`]. "We know what this is and
    /// it has no special shape" and "we do not know what this is" are different
    /// statements, and collapsing them would let an unrecognised role pass as a
    /// understood one.
    Unknown,
}

/// Sequence Ontology IRI to glyph.
///
/// Held as a table rather than a match so that adding a term is a data change,
/// and so the mapping can be inspected and tested as data.
const GLYPH_TABLE: &[(&str, Glyph)] = &[
    ("SO:0000167", Glyph::Promoter),
    ("SO:0000139", Glyph::RibosomeEntrySite),
    ("SO:0000316", Glyph::Cds),
    ("SO:0000141", Glyph::Terminator),
    ("SO:0000296", Glyph::OriginOfReplication),
    ("SO:0000704", Glyph::Generic), // gene
    ("SO:0000110", Glyph::Generic), // sequence_feature
];

/// Chooses a glyph for a Sequence Ontology role IRI.
///
/// Matches on the SO accession rather than the whole IRI, because the same term
/// appears with several prefixes in real files (`identifiers.org`,
/// `purl.obolibrary.org`, bare `SO:`). Requiring an exact IRI match would make
/// the renderer silently correct on one source and silently wrong on another.
#[must_use]
pub fn glyph_for_role(role_iri: &str) -> Glyph {
    let normalised = normalise_so_term(role_iri);
    GLYPH_TABLE
        .iter()
        .find(|(term, _)| *term == normalised)
        .map_or(Glyph::Unknown, |(_, g)| *g)
}

/// Extracts a `SO:NNNNNNN` accession from any of the common IRI spellings.
fn normalise_so_term(iri: &str) -> String {
    let tail = iri.rsplit(['/', '#']).next().unwrap_or(iri);
    // `SO_0000167` and `SO:0000167` both occur in the wild.
    let tail = tail.replace('_', ":");
    if tail.starts_with("SO:") {
        tail
    } else if tail.chars().all(|c| c.is_ascii_digit()) && !tail.is_empty() {
        format!("SO:{tail}")
    } else {
        tail
    }
}

impl Glyph {
    /// A short label for diagrams and accessible descriptions.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Promoter => "promoter",
            Self::RibosomeEntrySite => "RBS",
            Self::Cds => "CDS",
            Self::Terminator => "terminator",
            Self::OriginOfReplication => "ori",
            Self::Generic => "feature",
            Self::Unknown => "unknown",
        }
    }

    /// Whether this glyph stands for an unrecognised role.
    #[must_use]
    pub const fn is_unknown(self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// Whether the shape is drawn as lines rather than as a closed region.
    ///
    /// Promoters and terminators have no interior, so their category hue goes on
    /// the stroke. Everything else carries it in the fill.
    #[must_use]
    pub const fn is_stroke_only(self) -> bool {
        matches!(self, Self::Promoter | Self::Terminator)
    }

    /// The category hue for this glyph.
    ///
    /// Hue carries category and never confidence. That split is the reason the
    /// whole grammar survives greyscale: drop the colour and shape still says
    /// what a part is, while texture still says how much to trust it. If hue
    /// ever carried confidence, a greyscale print or a colour vision deficiency
    /// would erase the uncertainty and leave the prediction looking certain.
    #[must_use]
    pub const fn hue(self) -> Hue {
        match self {
            Self::Promoter => Hue {
                hex: "#3F8F5B",
                key: "promoter",
            },
            Self::RibosomeEntrySite => Hue {
                hex: "#D98A2B",
                key: "rbs",
            },
            Self::Cds => Hue {
                hex: "#2F6FA8",
                key: "cds",
            },
            Self::Terminator => Hue {
                hex: "#B0413E",
                key: "terminator",
            },
            Self::OriginOfReplication => Hue {
                hex: "#6B4FA8",
                key: "ori",
            },
            Self::Generic | Self::Unknown => Hue {
                hex: "#8A8A94",
                key: "neutral",
            },
        }
    }
}

/// A category colour, with a stable slug for naming pattern definitions.
///
/// The slug exists because a hatched or stippled fill is an SVG `<pattern>`, and
/// a pattern needs an id. Deriving the id from the hue rather than from the
/// feature keeps the defs block small and the output stable: two CDSes share one
/// pattern definition instead of generating one each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hue {
    /// Hex colour, including the leading `#`.
    pub hex: &'static str,
    /// Short identifier, safe for use in an XML id.
    pub key: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_the_standard_parts() {
        assert_eq!(
            glyph_for_role("https://identifiers.org/SO:0000167"),
            Glyph::Promoter
        );
        assert_eq!(
            glyph_for_role("https://identifiers.org/SO:0000139"),
            Glyph::RibosomeEntrySite
        );
        assert_eq!(
            glyph_for_role("https://identifiers.org/SO:0000316"),
            Glyph::Cds
        );
        assert_eq!(
            glyph_for_role("https://identifiers.org/SO:0000141"),
            Glyph::Terminator
        );
    }

    /// The same term appears with several prefixes in real files. Requiring an
    /// exact IRI match would make the renderer correct on one source and
    /// silently wrong on another.
    #[test]
    fn the_same_term_is_recognised_across_iri_spellings() {
        for spelling in [
            "https://identifiers.org/SO:0000167",
            "http://identifiers.org/so/SO:0000167",
            "http://purl.obolibrary.org/obo/SO_0000167",
            "SO:0000167",
        ] {
            assert_eq!(
                glyph_for_role(spelling),
                Glyph::Promoter,
                "failed to recognise the promoter term written as {spelling}"
            );
        }
    }

    /// An unrecognised role must be visibly unknown, not quietly generic.
    #[test]
    fn an_unrecognised_role_is_unknown_rather_than_generic() {
        let g = glyph_for_role("https://example.org/not-a-real-term");
        assert_eq!(g, Glyph::Unknown);
        assert!(
            g.is_unknown(),
            "an unrecognised role must be distinguishable from a recognised one with no \
             special shape; collapsing them lets a wrong diagram look right"
        );
        assert_ne!(g, Glyph::Generic);
    }

    #[test]
    fn every_table_entry_round_trips() {
        for (term, expected) in GLYPH_TABLE {
            assert_eq!(
                glyph_for_role(&format!("https://identifiers.org/{term}")),
                *expected,
                "table entry {term} does not resolve to its own glyph"
            );
        }
    }

    #[test]
    fn every_glyph_has_a_label() {
        for g in [
            Glyph::Promoter,
            Glyph::RibosomeEntrySite,
            Glyph::Cds,
            Glyph::Terminator,
            Glyph::OriginOfReplication,
            Glyph::Generic,
            Glyph::Unknown,
        ] {
            assert!(!g.label().is_empty(), "{g:?} has no label");
        }
    }
}
