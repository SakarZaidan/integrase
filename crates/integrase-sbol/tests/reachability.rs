// R-1.2 permits unwrap, expect and panic in test code.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Guards the reachability argument behind two documented advisory exceptions.
//!
//! `deny.toml` ignores RUSTSEC-2026-0194 and RUSTSEC-2026-0195, two
//! denial-of-service advisories against `quick-xml`. The justification has three
//! parts: the fix is not available downstream, a size cap mitigates the vector,
//! and **the vulnerable RDF/XML parser is not reachable from this crate's public
//! API**, which exposes Turtle only.
//!
//! That third part is the one that can quietly stop being true. Someone adds
//! RDF/XML import, the exception silently becomes load-bearing, and nobody
//! revisits it. This test makes that impossible: adding an RDF/XML entry point
//! fails the build until the exception is re-examined.

/// The public entry points this crate is known to expose.
///
/// Deliberately a hand-maintained list rather than reflection. The point is that
/// **a human has to edit this file** to add a parsing entry point, and editing
/// it means reading the note above.
const KNOWN_ENTRY_POINTS: &[&str] = &["from_turtle", "from_document"];

#[test]
fn no_rdf_xml_entry_point_exists_without_revisiting_the_advisory_exception() {
    let src = include_str!("../src/lib.rs");
    let import = include_str!("../src/import.rs");
    let combined = format!("{src}{import}");

    // Any public function whose name suggests it parses RDF/XML.
    let suspicious: Vec<&str> = combined
        .lines()
        .filter(|l| l.trim_start().starts_with("pub fn "))
        .filter(|l| {
            let lower = l.to_ascii_lowercase();
            lower.contains("rdf_xml") || lower.contains("rdfxml") || lower.contains("from_xml")
        })
        .collect();

    assert!(
        suspicious.is_empty(),
        "An RDF/XML entry point appeared:\n{}\n\n\
         That makes the vulnerable quick-xml path reachable, which invalidates the \
         reachability argument behind the RUSTSEC-2026-0194 and -0195 exceptions in \
         deny.toml. Re-examine those exceptions before adding this, and update both \
         this test and deny.toml's notes.",
        suspicious.join("\n")
    );
}

#[test]
fn the_documented_entry_points_still_exist() {
    // The mirror of the test above. If `from_turtle` were removed or renamed,
    // the guard above would pass vacuously while describing a surface that no
    // longer exists.
    let combined = format!(
        "{}{}",
        include_str!("../src/lib.rs"),
        include_str!("../src/import.rs")
    );
    for name in KNOWN_ENTRY_POINTS {
        assert!(
            combined.contains(&format!("pub fn {name}")),
            "documented entry point `{name}` is missing; the reachability guard is \
             describing an API that no longer exists"
        );
    }
}

#[test]
fn an_oversized_document_is_refused_before_parsing() {
    // The mitigation named in deny.toml. Built as a string that would be
    // expensive to parse, and must be rejected on length alone.
    let huge = "x".repeat(64 * 1024 * 1024 + 1);
    match integrase_sbol::from_turtle(&huge) {
        Err(integrase_sbol::SbolError::TooLarge { len, max }) => {
            assert_eq!(len, 64 * 1024 * 1024 + 1);
            assert_eq!(max, 64 * 1024 * 1024);
        }
        Err(other) => panic!("expected a size refusal, got {other:?}"),
        Ok(_) => panic!("an oversized document must be refused before parsing"),
    }
}
