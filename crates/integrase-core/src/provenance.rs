//! Where a value came from.
//!
//! Provenance travels with designs and predictions so that any result can be
//! traced to its inputs. Subplan 29 builds verifiable design records on top of
//! this; this module is the in-memory shape it serialises.

/// The origin of a design or a derived value.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum Provenance {
    /// Authored directly by a user in this system.
    Authored,
    /// Imported from an external source.
    Imported {
        /// Where it came from, as a URI or a file path.
        source: String,
        /// The format it arrived in, for example `SBOL3` or `GenBank`.
        format: String,
    },
    /// Derived from another design by a named operation.
    Derived {
        /// Identifier of the design this was derived from.
        parent: String,
        /// What was done, for example `reverse_complement` or `insert_feature`.
        operation: String,
    },
    /// Produced by a generative model.
    ///
    /// Kept distinct from [`Provenance::Authored`] deliberately. Subplan 26
    /// requires generated sequence to be permanently marked as generated, and a
    /// variant that could be confused with human authorship would defeat that.
    Generated {
        /// Identifier of the model that produced it.
        model_id: String,
    },
}
