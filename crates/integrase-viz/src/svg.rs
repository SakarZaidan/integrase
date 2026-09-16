//! Deterministic SVG rendering of a design.
//!
//! # Why SVG is a deliverable rather than a fallback
//!
//! Scientists put figures in papers, so an SVG export that carries the
//! confidence encoding means the uncertainty survives into publication, which is
//! where it reaches the widest audience.
//!
//! It also makes visual regression testing possible. SVG is text, so a rendering
//! change shows up as a readable diff. A GPU path would need image comparison
//! against a tolerance, and a tolerance loose enough to survive driver
//! differences is loose enough to miss a glyph drawn in the wrong place.
//!
//! # Determinism is a requirement, not a nicety
//!
//! The same design must produce byte-identical output on every run and every
//! machine, or the snapshot tests are worthless. That means features are sorted
//! before drawing rather than taken in import order, pattern definitions are
//! emitted in a sorted order rather than in hash-map order, coordinates are
//! formatted to a fixed number of decimal places, and nothing carries a
//! timestamp.
//!
//! # The two visual channels
//!
//! Shape and hue carry *what a part is*. Fill texture and border style carry
//! *how much the prediction about it can be trusted*, and they come from
//! [`crate::encoding_for`] alone. The channels never cross. Printed in
//! greyscale, category is still legible from shape and confidence is still
//! legible from texture.

use crate::{Border, ConfidenceEncoding, Fill, Glyph, IntervalCap, encoding_for, glyph_for_role};
use integrase_core::{Design, Strand};
use integrase_sim::DomainVerdict;

/// Ink colour for every stroke that is not carrying a category hue.
const INK: &str = "#16161A";

/// Layout and styling options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SvgOptions {
    /// Width of the drawing area in user units.
    pub width: u32,
    /// Height of the backbone row.
    pub track_height: u32,
    /// Padding around the drawing.
    pub padding: u32,
}

impl Default for SvgOptions {
    fn default() -> Self {
        Self {
            width: 800,
            track_height: 60,
            padding: 20,
        }
    }
}

/// Renders a design as SVG.
///
/// `verdict` supplies the confidence encoding. Pass
/// [`DomainVerdict::Uncalibrated`] when no prediction is attached, which draws
/// features as outline only: honest about the absence rather than showing them
/// as though they had been validated.
#[must_use]
pub fn render_svg(design: &Design, verdict: &DomainVerdict, opts: SvgOptions) -> String {
    let enc = encoding_for(verdict);
    let seq_len = design.sequence.len().max(1);
    let inner = f64::from(opts.width.saturating_sub(opts.padding * 2).max(1));
    let pad = f64::from(opts.padding);
    let height = opts.track_height + opts.padding * 2;
    let mid = pad + f64::from(opts.track_height) / 2.0;

    let x_of = |pos: usize| -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let frac = pos.min(seq_len) as f64 / seq_len as f64;
        pad + frac * inner
    };

    // Draw in sequence order, not import order. Two files describing the same
    // construct with the annotations listed differently must produce the same
    // figure, and the id is the final tiebreak so that co-located features of
    // the same span do not depend on input ordering either.
    let mut features: Vec<_> = design.features.iter().collect();
    features.sort_by(|a, b| {
        (a.location.start, a.location.end, &a.id.0).cmp(&(
            b.location.start,
            b.location.end,
            &b.id.0,
        ))
    });

    let mut s = String::new();
    s.push_str(&format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {height}" width="{}" height="{height}">"#,
        opts.width, opts.width));
    s.push('\n');

    s.push_str(&pattern_defs(&features, enc.fill));

    // A figure nobody can read with a screen reader excludes part of its
    // audience, and the confidence state belongs in the text alternative for
    // exactly the same reason it belongs in the drawing.
    s.push_str(&format!("  <title>{}</title>", escape(&design.id.0)));
    s.push('\n');
    let bp = design.sequence.len();
    let n = design.features.len();
    s.push_str(&format!(
        "  <desc>{bp} base {}, {n} annotated {}. Confidence: {}.</desc>",
        plural(bp, "pair", "pairs"),
        plural(n, "feature", "features"),
        describe_confidence(verdict)
    ));
    s.push('\n');

    s.push_str(&format!(r#"  <line x1="{pad:.2}" y1="{mid:.2}" x2="{:.2}" y2="{mid:.2}" stroke="{INK}" stroke-width="2"/>"#,
        pad + inner));
    s.push('\n');

    for f in features {
        let g = glyph_for_role(&f.role.0);
        let left = x_of(f.location.start);
        // A zero-length or sub-pixel feature still has to be visible. Clamping
        // the width is preferable to dropping it, because a feature that
        // silently vanishes from a figure is worse than one drawn slightly wide.
        let right = x_of(f.location.end).max(left + 6.0).min(pad + inner);
        s.push_str(&format!(
            "  {}",
            glyph_svg(
                g,
                left,
                right,
                mid,
                f.location.strand,
                &enc,
                f.name.as_deref()
            )
        ));
        s.push('\n');
    }

    s.push_str("</svg>\n");
    s
}

/// Emits `<pattern>` definitions for the hues actually used.
///
/// Textured fills need one pattern per hue, since an SVG pattern cannot inherit
/// the colour of the shape referencing it. Emitting only the hues present keeps
/// the defs block proportional to the figure, and sorting by key keeps it
/// deterministic.
fn pattern_defs(features: &[&integrase_core::Feature], fill: Fill) -> String {
    if !matches!(fill, Fill::Hatch45 | Fill::Stipple) {
        return String::new();
    }

    // Only glyphs that actually take a fill. A promoter is drawn as lines, so a
    // pattern named after it would sit in the defs block referenced by nothing,
    // and dead markup in a figure is a slow leak: nobody notices it, and it
    // grows every time a glyph is added.
    let mut keys: Vec<(&str, &str)> = features
        .iter()
        .map(|f| glyph_for_role(&f.role.0))
        .filter(|g| !g.is_stroke_only())
        .map(|g| {
            let h = g.hue();
            (h.key, h.hex)
        })
        .collect();
    keys.sort_unstable();
    keys.dedup();

    if keys.is_empty() {
        return String::new();
    }

    let mut s = String::from("  <defs>\n");
    for (key, hex) in keys {
        match fill {
            Fill::Hatch45 => {
                s.push_str(&format!(r#"    <pattern id="hatch-{key}" width="8" height="8" patternTransform="rotate(45)" patternUnits="userSpaceOnUse"><rect width="8" height="8" fill="{hex}"/><line x1="0" y1="0" x2="0" y2="8" stroke="{INK}" stroke-width="2"/></pattern>"#));
                s.push('\n');
            }
            Fill::Stipple => {
                s.push_str(&format!(r#"    <pattern id="stipple-{key}" width="6" height="6" patternUnits="userSpaceOnUse"><rect width="6" height="6" fill="{hex}" fill-opacity="0.25"/><circle cx="1.5" cy="1.5" r="1" fill="{INK}"/></pattern>"#));
                s.push('\n');
            }
            Fill::Solid | Fill::None => {}
        }
    }
    s.push_str("  </defs>\n");
    s
}

/// Picks a singular or plural noun.
///
/// The text alternative is the version a screen reader speaks aloud, so "1
/// annotated features" is read out as written. Grammar is not decoration here.
const fn plural(n: usize, one: &'static str, many: &'static str) -> &'static str {
    if n == 1 { one } else { many }
}

/// The confidence state in words, for the text alternative.
const fn describe_confidence(v: &DomainVerdict) -> &'static str {
    match v {
        DomainVerdict::InDistribution { .. } => "inside the model's validated range",
        DomainVerdict::Marginal { .. } => "near the edge of the model's validated range",
        DomainVerdict::OutOfDistribution { .. } => "outside the model's validated range",
        // Both the explicit variant and any future one. `DomainVerdict` is
        // `#[non_exhaustive]`, and an unrecognised state is one this renderer
        // cannot vouch for, so it describes itself as such rather than guessing.
        _ => "not calibrated",
    }
}

/// The `fill` attribute for a glyph under a confidence encoding.
fn fill_attr(fill: Fill, g: Glyph) -> String {
    let hue = g.hue();
    match fill {
        Fill::Solid => format!(r#"fill="{}""#, hue.hex),
        Fill::Hatch45 => format!(r#"fill="url(#hatch-{})""#, hue.key),
        Fill::Stipple => format!(r#"fill="url(#stipple-{})""#, hue.key),
        Fill::None => r#"fill="none""#.to_owned(),
    }
}

/// The stroke attributes for a glyph under a confidence encoding.
///
/// Dash pattern is the confidence channel and colour is the category channel, so
/// the two are set independently and a greyscale conversion loses only the
/// second.
fn stroke_attr(border: Border, g: Glyph) -> String {
    let colour = if g.is_stroke_only() { g.hue().hex } else { INK };
    let dash = match border {
        Border::Solid => "",
        Border::Dashed => r#" stroke-dasharray="6 3""#,
        Border::Dotted => r#" stroke-dasharray="1 3""#,
    };
    format!(r#"stroke="{colour}" stroke-width="1.5"{dash}"#)
}

/// Draws one glyph into the span `[left, right]`.
///
/// Shapes follow SBOL Visual conventions: a bent arrow for a promoter, a dome
/// for a ribosome entry site, an arrow-shaped box for a CDS, a T-bar for a
/// terminator, a circle for an origin.
///
/// Strand changes which way the shape points, never where it sits. A
/// reverse-strand feature occupies the same span as a forward one at the same
/// coordinates, because the span is a fact about the sequence and the direction
/// is a fact about the feature.
fn glyph_svg(
    g: Glyph,
    left: f64,
    right: f64,
    mid: f64,
    strand: Strand,
    enc: &ConfidenceEncoding,
    name: Option<&str>,
) -> String {
    let fill = fill_attr(enc.fill, g);
    let stroke = stroke_attr(enc.border, g);
    let h = 18.0;
    let half = h / 2.0;
    let top = mid - h;
    let forward = matches!(strand, Strand::Forward);
    // 5' and 3' ends of the feature as drawn. On the reverse strand they swap,
    // which is the whole of what strand means visually.
    let (from, to) = if forward {
        (left, right)
    } else {
        (right, left)
    };
    let step = if forward { 1.0_f64 } else { -1.0_f64 };

    let label = name.map_or_else(
        || format!("<title>{}</title>", escape(g.label())),
        |n| format!("<title>{} ({})</title>", escape(n), escape(g.label())),
    );

    let body = match g {
        // Bent arrow: up at the 5' end, across, arrowhead at the 3' end.
        Glyph::Promoter => format!(
            r#"<path d="M {from:.2} {mid:.2} L {from:.2} {top:.2} L {to:.2} {top:.2} M {barb:.2} {up:.2} L {to:.2} {top:.2} L {barb:.2} {down:.2}" fill="none" {stroke} stroke-linejoin="round"/>"#,
            barb = to - 6.0 * step,
            up = top - 4.0,
            down = top + 4.0,
        ),
        // Dome over the backbone. Symmetric, so strand does not change it.
        Glyph::RibosomeEntrySite => format!(
            r#"<path d="M {left:.2} {mid:.2} A {r:.2} {r:.2} 0 0 1 {right:.2} {mid:.2} Z" {fill} {stroke}/>"#,
            r = (right - left) / 2.0,
        ),
        // Arrow-shaped box. The point is at the 3' end, and the shoulder never
        // crosses the 5' end however short the feature is.
        Glyph::Cds => {
            let shoulder = if forward {
                (to - 8.0).max(from + 1.0)
            } else {
                (to + 8.0).min(from - 1.0)
            };
            format!(
                r#"<path d="M {from:.2} {t:.2} L {shoulder:.2} {t:.2} L {to:.2} {mid:.2} L {shoulder:.2} {b:.2} L {from:.2} {b:.2} Z" {fill} {stroke}/>"#,
                t = mid - half,
                b = mid + half,
            )
        }
        // T-bar, centred on the span. Terminators are not directional.
        Glyph::Terminator => format!(
            r#"<path d="M {cx:.2} {mid:.2} L {cx:.2} {top:.2} M {l:.2} {top:.2} L {r:.2} {top:.2}" fill="none" {stroke}/>"#,
            cx = f64::midpoint(left, right),
            l = f64::midpoint(left, right) - 8.0,
            r = f64::midpoint(left, right) + 8.0,
        ),
        Glyph::OriginOfReplication => format!(
            r#"<circle cx="{cx:.2}" cy="{mid:.2}" r="{r:.2}" {fill} {stroke}/>"#,
            cx = f64::midpoint(left, right),
            r = ((right - left) / 2.0).min(9.0),
        ),
        Glyph::Generic => format!(
            r#"<rect x="{left:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}" {fill} {stroke}/>"#,
            y = mid - half,
            w = right - left,
        ),
        // A box struck through. The slash is the point: an unrecognised role
        // must not be mistaken for a recognised one that happens to lack a
        // distinctive shape, and the difference has to be visible in the figure,
        // not only in the tooltip.
        //
        // The mark is a shape rather than a border style because border style is
        // already spent on confidence. Adding a second meaning to it would make
        // an unrecognised role in-distribution look like a recognised role out
        // of distribution.
        Glyph::Unknown => format!(
            r#"<rect x="{left:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}" {fill} {stroke}/><path d="M {left:.2} {b:.2} L {right:.2} {y:.2}" fill="none" stroke="{INK}" stroke-width="1.5"/>"#,
            y = mid - half,
            b = mid + half,
            w = right - left,
        ),
    };

    format!("<g>{body}{label}</g>")
}

/// Escapes text for inclusion in SVG character data.
///
/// Design identifiers are frequently full IRIs and feature names come from
/// imported files, so neither can be trusted to be XML-safe.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

/// Whether an interval bar should be drawn for this confidence state.
///
/// Exposed so that a caller rendering a prediction alongside a design uses the
/// same rule the encoding does instead of deciding independently.
#[must_use]
pub const fn draws_interval_bar(enc: &ConfidenceEncoding) -> bool {
    !matches!(enc.interval, IntervalCap::None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_covers_the_five_xml_entities() {
        assert_eq!(escape("<&>\"'"), "&lt;&amp;&gt;&quot;&apos;");
    }

    #[test]
    fn hue_never_depends_on_confidence() {
        // The same glyph must keep its category colour across every confidence
        // state. If this fails, greyscale printing loses category as well as
        // confidence, and the encoding stops being two independent channels.
        let g = Glyph::Cds;
        let hex = g.hue().hex;
        for fill in [Fill::Solid, Fill::Hatch45, Fill::Stipple] {
            let attr = fill_attr(fill, g);
            assert!(
                attr.contains(hex) || attr.contains(g.hue().key),
                "{fill:?} lost the category hue"
            );
        }
    }

    #[test]
    fn dash_pattern_never_depends_on_category() {
        // The mirror of the test above: border style is the confidence channel
        // only. Two different glyphs at the same confidence must dash alike.
        let a = stroke_attr(Border::Dashed, Glyph::Cds);
        let b = stroke_attr(Border::Dashed, Glyph::Generic);
        let dash = |s: &str| s.split("stroke-dasharray").nth(1).map(str::to_owned);
        assert_eq!(dash(&a), dash(&b));
    }

    #[test]
    fn uncalibrated_draws_no_interval_bar() {
        assert!(!draws_interval_bar(&encoding_for(
            &DomainVerdict::Uncalibrated
        )));
    }
}
