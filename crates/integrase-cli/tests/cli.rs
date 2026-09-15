// R-1.2 permits unwrap, expect and panic in test code.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! End-to-end tests for the command-line interface.
//!
//! These run the built binary rather than calling library functions, because the
//! things most worth checking here are properties of the *process*: exit codes,
//! what lands on stdout versus stderr, and whether a refusal leaks partial
//! output. None of that is visible from a unit test.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn binary() -> PathBuf {
    // The test binary lives next to the CLI binary in the same profile dir.
    let mut p = std::env::current_exe().expect("test binary path");
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    p.join(if cfg!(windows) {
        "integrase.exe"
    } else {
        "integrase"
    })
}

fn run(args: &[&str]) -> Output {
    Command::new(binary())
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {}: {e}", binary().display()))
}

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/sbol3")
}

/// A fixture that actually yields a design with sequence.
fn fixture_with_a_design() -> PathBuf {
    corpus().join("bba_f2620_popsreceiver.pySBOL3.expected.ttl")
}

// --- scope -------------------------------------------------------------------

#[test]
fn scope_states_the_limits_and_succeeds() {
    let out = run(&["scope"]);
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    for phrase in [
        "edit genomes",
        "not measurements",
        "experimental validation",
    ] {
        assert!(s.contains(phrase), "scope output must mention {phrase:?}");
    }
}

// --- predict -----------------------------------------------------------------

#[test]
fn predict_reports_the_absent_interval_with_its_reason() {
    let out = run(&["predict", "TTTTTTAGGAGGTTTTT"]);
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains("no calibration evidence"),
        "an absent interval must say why; a blank field reads as missing data"
    );
    assert!(s.contains("uncalibrated"));
}

#[test]
fn predict_warns_when_coverage_is_unreliable() {
    let s = String::from_utf8_lossy(&run(&["predict", "TTTTTTAGGAGGTTTTT"]).stdout).to_string();
    assert!(
        s.contains("plausible"),
        "the user must be told how to read a number with no coverage guarantee"
    );
}

#[test]
fn predict_names_the_host_mismatch_in_plain_words() {
    let out = run(&[
        "predict",
        "TTTTTTAGGAGGTTTTT",
        "--host",
        "Paracoccus denitrificans",
    ]);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("OUT OF DISTRIBUTION"));
    assert!(
        s.contains("differs from the fitted host"),
        "the reason must be readable prose, not a Debug dump: {s}"
    );
    assert!(
        !s.contains("HostMismatch {"),
        "Debug formatting is developer output and must not reach a user: {s}"
    );
}

#[test]
fn a_malformed_start_codon_fails_with_a_message_on_stderr() {
    let out = run(&["predict", "TTTTTTAGGAGGTTTTT", "--start-codon", "AT"]);
    assert!(!out.status.success(), "a bad argument must not exit zero");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("3 bases"),
        "the error must say what was wrong: {err}"
    );
}

// --- search ------------------------------------------------------------------

#[test]
fn search_finds_restriction_sites_on_both_strands() {
    let out = run(&["search", "AAAGAATTCAAA"]);
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("EcoRI"), "GAATTC is EcoRI: {s}");
    assert!(
        s.contains("Forward") && s.contains("Reverse"),
        "both strands must be reported"
    );
}

#[test]
fn search_accepts_an_explicit_motif() {
    let s = String::from_utf8_lossy(&run(&["search", "AAAGAATTCAAA", "--motif", "GAATTC"]).stdout)
        .to_string();
    assert!(s.contains("match(es) for GAATTC"), "{s}");
}

// --- import ------------------------------------------------------------------

#[test]
fn import_reports_designs_and_conversion_notes() {
    let f = fixture_with_a_design();
    let out = run(&["import", f.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("design(s) imported"));
    assert!(
        s.contains("conversion note") || s.contains("nothing was dropped"),
        "the importer must state what it could not carry, either way: {s}"
    );
}

#[test]
fn import_of_a_missing_file_fails_rather_than_reporting_zero_designs() {
    let out = run(&["import", "/nonexistent/file.ttl"]);
    assert!(
        !out.status.success(),
        "a missing file must be an error, not an empty success"
    );
}

// --- export: the gate --------------------------------------------------------

/// **The property that matters most in this file.**
///
/// With no screening backend configured, a synthesis-bound export must refuse,
/// exit non-zero, and write **nothing** to stdout. A script doing
/// `integrase export ... > order.fasta` must end up with an empty file and a
/// failed command, never a partial or unscreened one.
#[test]
fn export_refuses_without_a_backend_and_leaks_nothing_to_stdout() {
    let f = fixture_with_a_design();
    let out = run(&["export", f.to_str().unwrap()]);

    assert!(
        !out.status.success(),
        "an unscreened export must not exit zero"
    );
    assert!(
        out.stdout.is_empty(),
        "refusal must produce no stdout at all; a script redirecting output would \
         otherwise get a partial or unscreened file. Got {} bytes.",
        out.stdout.len()
    );

    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("No screening backend is configured"),
        "the refusal must explain itself: {err}"
    );
    assert!(
        err.contains("unaffected"),
        "it must also say what still works, or it reads as a broken tool: {err}"
    );
}
