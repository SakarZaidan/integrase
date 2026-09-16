// R-1.2 permits unwrap, expect and panic in test code. Integration tests are a
// separate crate, so the crate-level `cfg_attr(test, ...)` allowance in lib.rs
// does not reach here and has to be declared again.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Golden-file snapshot tests for SVG output.
//!
//! # Why golden files and not an image comparison
//!
//! SVG is text, so a rendering change arrives as a readable diff a reviewer can
//! actually judge. An image comparison needs a similarity tolerance, and a
//! tolerance loose enough to survive font rendering differences is loose enough
//! to miss a glyph drawn in the wrong place.
//!
//! # Why no snapshot library
//!
//! Adding a dependency to compare two strings is not a trade worth making, and
//! every dependency here goes through an advisory check first (R-3.2). Twenty
//! lines of harness is cheaper than that review, and it keeps the update path
//! explicit: `UPDATE_GOLDEN=1 cargo test -p integrase-viz`.

use integrase_core::{
    Alphabet, Design, DesignId, Feature, FeatureId, Location, OntologyTerm, Provenance, Sequence,
    Strand,
};
use integrase_sim::{DomainReason, DomainVerdict};
use integrase_viz::{SvgOptions, render_svg};

/// Compares `actual` against the committed golden file, or rewrites it when
/// `UPDATE_GOLDEN` is set.
///
/// On mismatch the actual output is written next to the golden file so the
/// failure can be inspected with an ordinary diff rather than by reading a
/// multi-kilobyte assertion message.
fn assert_golden(name: &str, actual: &str) {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");
    let path = format!("{dir}/{name}.svg");

    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&path, actual).expect("write golden");
        return;
    }

    let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("missing golden file {path}: {e}. Run UPDATE_GOLDEN=1 cargo test -p integrase-viz")
    });

    if expected != actual {
        let actual_path = format!("{dir}/{name}.actual.svg");
        std::fs::write(&actual_path, actual).expect("write actual");
        panic!("SVG output changed. diff {path} {actual_path}");
    }
}

/// A small construct with one of each glyph shape, including a reverse-strand
/// CDS so orientation is covered.
fn fixture() -> Design {
    let mut features = Vec::new();
    let mut push = |id: &str, start, end, strand, role: OntologyTerm, name: &str| {
        features.push(Feature {
            id: FeatureId(id.into()),
            location: Location { start, end, strand },
            role,
            name: Some(name.into()),
            children: Vec::new(),
        });
    };
    push(
        "p1",
        0,
        35,
        Strand::Forward,
        OntologyTerm::promoter(),
        "J23100",
    );
    push("r1", 40, 55, Strand::Forward, OntologyTerm::rbs(), "B0034");
    push("c1", 60, 180, Strand::Forward, OntologyTerm::cds(), "GFP");
    push(
        "c2",
        190,
        280,
        Strand::Reverse,
        OntologyTerm::cds(),
        "mCherry",
    );
    push(
        "t1",
        290,
        320,
        Strand::Forward,
        OntologyTerm::terminator(),
        "B0015",
    );
    // An IRI the glyph table does not carry, so the unknown path is rendered.
    push(
        "x1",
        330,
        360,
        Strand::Forward,
        OntologyTerm("https://identifiers.org/SO:9999999".into()),
        "mystery",
    );

    Design {
        id: DesignId("urn:integrase:test:reporter-cassette".into()),
        sequence: Sequence::new(vec![b'A'; 400], Alphabet::Dna).expect("valid fixture"),
        features,
        provenance: Provenance::Authored,
    }
}

#[test]
fn in_distribution_renders_solid() {
    let svg = render_svg(
        &fixture(),
        &DomainVerdict::InDistribution { similarity: 0.94 },
        SvgOptions::default(),
    );
    assert_golden("in_distribution", &svg);
}

#[test]
fn out_of_distribution_renders_stippled_and_dashed() {
    let svg = render_svg(
        &fixture(),
        &DomainVerdict::OutOfDistribution {
            reason: DomainReason::LengthOutOfRange { length: 3 },
        },
        SvgOptions::default(),
    );
    assert_golden("out_of_distribution", &svg);
}

#[test]
fn uncalibrated_renders_outline_only() {
    let svg = render_svg(
        &fixture(),
        &DomainVerdict::Uncalibrated,
        SvgOptions::default(),
    );
    assert_golden("uncalibrated", &svg);
}

/// The property that matters most: the same input renders byte-identically.
///
/// A snapshot test that passes only because it happened to run twice in the
/// same process order is worthless, so this asserts determinism directly
/// instead of inferring it.
#[test]
fn rendering_is_deterministic() {
    let d = fixture();
    let v = DomainVerdict::InDistribution { similarity: 0.9 };
    let first = render_svg(&d, &v, SvgOptions::default());
    for _ in 0..8 {
        assert_eq!(first, render_svg(&d, &v, SvgOptions::default()));
    }
}

/// Feature order in the input must not change the output.
///
/// `Design::features` preserves import order, so two files describing the same
/// construct with the annotations listed differently would otherwise produce
/// different figures.
#[test]
fn input_order_does_not_affect_output() {
    let a = fixture();
    let mut b = fixture();
    b.features.reverse();
    let v = DomainVerdict::Uncalibrated;
    assert_eq!(
        render_svg(&a, &v, SvgOptions::default()),
        render_svg(&b, &v, SvgOptions::default())
    );
}

/// Every confidence state must produce visibly different markup.
///
/// The encoding table already asserts the states are distinct as values. This
/// asserts the renderer actually spends them, which is the part that could
/// silently regress.
#[test]
fn confidence_states_render_differently() {
    let d = fixture();
    let renders = [
        render_svg(
            &d,
            &DomainVerdict::InDistribution { similarity: 0.9 },
            SvgOptions::default(),
        ),
        render_svg(
            &d,
            &DomainVerdict::Marginal {
                similarity: 0.5,
                reason: DomainReason::LengthOutOfRange { length: 3 },
            },
            SvgOptions::default(),
        ),
        render_svg(
            &d,
            &DomainVerdict::OutOfDistribution {
                reason: DomainReason::LengthOutOfRange { length: 3 },
            },
            SvgOptions::default(),
        ),
        render_svg(&d, &DomainVerdict::Uncalibrated, SvgOptions::default()),
    ];
    for (i, a) in renders.iter().enumerate() {
        for b in renders.iter().skip(i + 1) {
            assert_ne!(a, b);
        }
    }
}

/// Output must be well-formed XML.
///
/// Feature names and design identifiers come from imported files, so an
/// unescaped `&` in a name would produce a file no viewer will open. The
/// fixture here carries characters chosen to break naive concatenation.
#[test]
fn hostile_names_do_not_break_the_document() {
    let mut d = fixture();
    d.id = DesignId("a<b>&c\"d'e".into());
    d.features[0].name = Some("P<sub>lac</sub> & \"friends\"".into());
    let svg = render_svg(&d, &DomainVerdict::Uncalibrated, SvgOptions::default());

    assert!(!svg.contains("a<b>"), "identifier was not escaped");
    assert!(svg.contains("a&lt;b&gt;&amp;c&quot;d&apos;e"));
    assert!(svg.contains("&amp; &quot;friends&quot;"));

    // Outside the tag-opening positions the renderer itself emits, no raw `<`
    // should survive. Counting tags is the cheap approximation of a parse.
    let opens = svg.matches('<').count();
    let closes = svg.matches('>').count();
    assert_eq!(opens, closes, "unbalanced angle brackets");
}

/// A feature running to the end of the sequence must not draw past the track,
/// and a zero-length feature must still be visible.
#[test]
fn degenerate_locations_stay_in_bounds() {
    let mut d = fixture();
    d.features = vec![
        Feature {
            id: FeatureId("full".into()),
            location: Location {
                start: 0,
                end: 400,
                strand: Strand::Forward,
            },
            role: OntologyTerm::cds(),
            name: None,
            children: Vec::new(),
        },
        Feature {
            id: FeatureId("empty".into()),
            location: Location {
                start: 200,
                end: 200,
                strand: Strand::Forward,
            },
            role: OntologyTerm::promoter(),
            name: None,
            children: Vec::new(),
        },
    ];
    let svg = render_svg(&d, &DomainVerdict::Uncalibrated, SvgOptions::default());
    assert!(svg.contains("<g>"));

    // Every x coordinate the renderer emits must sit inside the viewBox.
    for cap in svg.split('"').filter(|s| s.parse::<f64>().is_ok()) {
        let v: f64 = cap.parse().expect("filtered above");
        assert!(
            (-1.0..=801.0).contains(&v),
            "coordinate {v} escaped the canvas"
        );
    }
}

/// An empty design still renders a valid document rather than dividing by zero.
#[test]
fn empty_design_renders() {
    let d = Design {
        id: DesignId("empty".into()),
        sequence: Sequence::new(Vec::new(), Alphabet::Dna).expect("empty is valid"),
        features: Vec::new(),
        provenance: Provenance::Authored,
    };
    let svg = render_svg(&d, &DomainVerdict::Uncalibrated, SvgOptions::default());
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>\n"));
    assert!(!svg.contains("NaN"), "degenerate scale produced NaN");
    assert!(!svg.contains("inf"));
}

/// Every pattern the document defines is referenced, and every pattern it
/// references is defined.
///
/// A dangling `url(#...)` renders as black in some viewers and as nothing in
/// others, so it is a real rendering bug rather than untidiness. An unreferenced
/// definition is the cheaper half of the same mistake, and both are invisible
/// until someone opens the file.
#[test]
fn pattern_definitions_and_references_agree() {
    let d = fixture();
    for verdict in [
        DomainVerdict::InDistribution { similarity: 0.9 },
        DomainVerdict::Marginal {
            similarity: 0.5,
            reason: DomainReason::LengthOutOfRange { length: 3 },
        },
        DomainVerdict::OutOfDistribution {
            reason: DomainReason::LengthOutOfRange { length: 3 },
        },
        DomainVerdict::Uncalibrated,
    ] {
        let svg = render_svg(&d, &verdict, SvgOptions::default());

        let mut defined: Vec<&str> = svg
            .match_indices(r#"<pattern id=""#)
            .map(|(i, m)| {
                let rest = &svg[i + m.len()..];
                &rest[..rest.find('"').expect("id is quoted")]
            })
            .collect();
        let mut referenced: Vec<&str> = svg
            .match_indices("url(#")
            .map(|(i, m)| {
                let rest = &svg[i + m.len()..];
                &rest[..rest.find(')').expect("url is closed")]
            })
            .collect();

        defined.sort_unstable();
        defined.dedup();
        referenced.sort_unstable();
        referenced.dedup();

        assert_eq!(defined, referenced, "pattern mismatch for {verdict:?}");
    }
}

/// A reverse-strand feature occupies the same span as a forward one.
///
/// Strand changes which way a glyph points, never where it sits. An earlier
/// version of this renderer mirrored the shape about its own start coordinate,
/// which drew reverse features at entirely the wrong place on the backbone. The
/// figure looked plausible, which is what made it dangerous.
#[test]
fn strand_changes_direction_not_position() {
    let span = |strand| {
        let d = Design {
            id: DesignId("s".into()),
            sequence: Sequence::new(vec![b'A'; 400], Alphabet::Dna).expect("valid"),
            features: vec![Feature {
                id: FeatureId("c".into()),
                location: Location {
                    start: 190,
                    end: 280,
                    strand,
                },
                role: OntologyTerm::cds(),
                name: None,
                children: Vec::new(),
            }],
            provenance: Provenance::Authored,
        };
        let svg = render_svg(&d, &DomainVerdict::Uncalibrated, SvgOptions::default());
        let xs: Vec<f64> = svg
            .split(&[' ', 'L', 'M', 'Z'][..])
            .filter_map(|t| t.parse::<f64>().ok())
            .collect();
        // Path coordinates alternate x, y. Take the extremes of the x values
        // inside the feature path, skipping the backbone line above it.
        let lo = xs
            .iter()
            .copied()
            .filter(|v| *v > 100.0)
            .fold(f64::MAX, f64::min);
        let hi = xs
            .iter()
            .copied()
            .filter(|v| *v < 700.0)
            .fold(f64::MIN, f64::max);
        (lo, hi)
    };

    let (f_lo, f_hi) = span(Strand::Forward);
    let (r_lo, r_hi) = span(Strand::Reverse);
    assert!(
        (f_lo - r_lo).abs() < 0.01,
        "left edge moved: {f_lo} vs {r_lo}"
    );
    assert!(
        (f_hi - r_hi).abs() < 0.01,
        "right edge moved: {f_hi} vs {r_hi}"
    );

    // And the direction genuinely differs, so this test cannot pass by the
    // renderer ignoring strand entirely.
    let d = |strand| {
        let design = Design {
            id: DesignId("s".into()),
            sequence: Sequence::new(vec![b'A'; 400], Alphabet::Dna).expect("valid"),
            features: vec![Feature {
                id: FeatureId("c".into()),
                location: Location {
                    start: 190,
                    end: 280,
                    strand,
                },
                role: OntologyTerm::cds(),
                name: None,
                children: Vec::new(),
            }],
            provenance: Provenance::Authored,
        };
        render_svg(&design, &DomainVerdict::Uncalibrated, SvgOptions::default())
    };
    assert_ne!(d(Strand::Forward), d(Strand::Reverse));
}
