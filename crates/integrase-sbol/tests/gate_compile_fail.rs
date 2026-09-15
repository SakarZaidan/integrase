//! Compile-time proof of the export gate.
//!
//! Subplan 17 names this as the only test that verifies the **type-level**
//! guarantee rather than runtime behaviour. Every other screening test shows
//! that a cleared design exports; none of them can show that an uncleared one
//! *cannot*, because the code expressing that attempt does not compile and so
//! cannot be written inside a normal test.
//!
//! Without this, a refactor could make `Cleared` constructible elsewhere, or
//! loosen `export_for_synthesis` to accept a bare `Design`, and every runtime
//! test would still pass.

#[test]
fn an_unscreened_design_cannot_reach_a_synthesis_export() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/unscreened_export.rs");
}
