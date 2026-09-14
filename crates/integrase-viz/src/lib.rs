//! Rendering.
//!
//! Two backends over one scene description: `wgpu` for interactive use, SVG for
//! figures, documentation, and snapshot testing.
//!
//! SVG is a deliverable rather than a fallback. Scientists put figures in
//! papers, so an SVG export that carries the confidence grammar means the
//! uncertainty encoding survives into publication. It is also deterministic and
//! text-diffable, which makes visual regression testing possible where a GPU
//! path would need image comparison with a tolerance nobody trusts.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use integrase_sim::DomainVerdict;

/// How a feature's fill is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    /// Solid, full opacity.
    Solid,
    /// 45-degree hatching.
    Hatch45,
    /// Low-density stipple.
    Stipple,
    /// No fill; outline only.
    None,
}

/// How a feature's border is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Border {
    /// Solid line.
    Solid,
    /// Dashed line.
    Dashed,
    /// Dotted line.
    Dotted,
}

/// How an interval bar's end caps are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntervalCap {
    /// Solid caps: a narrow, confident interval.
    Solid,
    /// Hatched caps: provisional.
    Hatched,
    /// Open caps: a guess, and openly so.
    Open,
    /// No bar at all.
    None,
}

/// The complete visual encoding of a confidence state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfidenceEncoding {
    /// Fill treatment.
    pub fill: Fill,
    /// Border treatment.
    pub border: Border,
    /// Interval bar treatment.
    pub interval: IntervalCap,
}

/// Maps a domain verdict to its visual encoding.
///
/// **The only place visual confidence is decided.** A single function so that it
/// cannot drift between views, and so snapshot tests cover all of it. There is
/// no code path where a developer chooses how confident something looks.
///
/// Texture carries confidence and hue carries category, which is not a stylistic
/// preference. Measured contrast ratios between the four semantic colours run
/// 1.14:1 to 1.70:1, all far under 3:1, so those states are not reliably
/// distinguishable by luminance and hue alone cannot carry them for anyone with
/// a colour vision deficiency. The whole grammar survives greyscale, which is
/// the test.
#[must_use]
pub const fn encoding_for(verdict: &DomainVerdict) -> ConfidenceEncoding {
    match verdict {
        DomainVerdict::InDistribution { .. } => ConfidenceEncoding {
            fill: Fill::Solid,
            border: Border::Solid,
            interval: IntervalCap::Solid,
        },
        DomainVerdict::Marginal { .. } => ConfidenceEncoding {
            fill: Fill::Hatch45,
            border: Border::Solid,
            interval: IntervalCap::Hatched,
        },
        DomainVerdict::OutOfDistribution { .. } => ConfidenceEncoding {
            fill: Fill::Stipple,
            border: Border::Dashed,
            interval: IntervalCap::Open,
        },
        // No interval bar. A model with no calibration evidence has no honest
        // interval to draw, and drawing a wide one would imply a quantified
        // uncertainty we have not earned. Absence is the truthful rendering.
        DomainVerdict::Uncalibrated => ConfidenceEncoding {
            fill: Fill::None,
            border: Border::Dotted,
            interval: IntervalCap::None,
        },
        // `DomainVerdict` is `#[non_exhaustive]`, so a future variant must land
        // somewhere. It lands on the *least* confident rendering, not the most.
        //
        // This is the fail-safe direction: a confidence state this renderer does
        // not recognise is, definitionally, one it cannot vouch for. Defaulting
        // to solid fill would draw an unknown state as validated, which is the
        // single worst rendering error this system could make.
        _ => ConfidenceEncoding {
            fill: Fill::None,
            border: Border::Dotted,
            interval: IntervalCap::None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use integrase_sim::DomainReason;

    #[test]
    fn every_verdict_maps_to_a_distinct_encoding() {
        let encodings = [
            encoding_for(&DomainVerdict::InDistribution { similarity: 0.9 }),
            encoding_for(&DomainVerdict::Marginal {
                similarity: 0.5,
                reason: DomainReason::LengthOutOfRange { length: 3 },
            }),
            encoding_for(&DomainVerdict::OutOfDistribution {
                reason: DomainReason::LengthOutOfRange { length: 3 },
            }),
            encoding_for(&DomainVerdict::Uncalibrated),
        ];
        for (i, a) in encodings.iter().enumerate() {
            for b in encodings.iter().skip(i + 1) {
                assert_ne!(a, b, "confidence states must be visually distinguishable");
            }
        }
    }

    #[test]
    fn uncalibrated_draws_no_interval_bar() {
        assert_eq!(
            encoding_for(&DomainVerdict::Uncalibrated).interval,
            IntervalCap::None
        );
    }
}
