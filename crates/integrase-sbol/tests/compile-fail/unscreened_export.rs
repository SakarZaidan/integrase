//! An unscreened design must not reach a synthesis-bound export.
//!
//! This file is expected to FAIL to compile. If it ever builds, tier-0 rule
//! R-0.2 has been broken and the screening gate is bypassable.

use integrase_core::{Alphabet, Design, DesignId, Provenance, Sequence};
use integrase_sbol::{export_for_synthesis, VendorFormat};

fn main() {
    let design = Design {
        id: DesignId("sneaky".into()),
        sequence: Sequence::new(b"ACGTACGTACGT".to_vec(), Alphabet::DnaIupac).unwrap(),
        features: Vec::new(),
        provenance: Provenance::Authored,
    };

    // Must not compile: `export_for_synthesis` takes `&Cleared<Design>`, and
    // `Cleared` is constructible only inside `integrase-screen`.
    let _ = export_for_synthesis(&design, VendorFormat::Fasta);
}
