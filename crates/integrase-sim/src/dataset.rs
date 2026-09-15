//! Loading measured expression data for calibration.
//!
//! # Why there is no data in this repository
//!
//! Calibration needs sequences with measured expression values. The canonical
//! source is Kosuri et al. 2013 (PNAS), which measured 12,653 promoter and RBS
//! combinations in *E. coli*. Located precisely: PMC3752251, supplementary
//! Datasets S1, S2 and S3.
//!
//! It is **not vendored**, for two reasons, and the second one is the binding
//! one:
//!
//! 1. PMC serves an interstitial page in place of the file for automated
//!    requests. Working around an access control the host deliberately placed
//!    there is not something this project does.
//! 2. **The redistribution terms are unresolved.** The article is copyright the
//!    National Academy of Sciences. Free to read is not the same as free to
//!    redistribute, and subplan 07's R-3.3 holds that data licences are checked
//!    like code licences. Vendoring it into an Apache-2.0 repository without
//!    establishing that would be exactly the kind of thing that policy exists to
//!    prevent.
//!
//! So the data is loaded from a local path the operator provides, the directory
//! is gitignored, and `docs/calibration-data.md` documents how to obtain it.
//!
//! # What happens when it is absent
//!
//! Nothing silently passes. Models report [`crate::DomainVerdict::Uncalibrated`]
//! and offer no interval, which is the truthful state rather than a placeholder.
//! Calibration tests that need the data report that they were skipped **and
//! why**, rather than passing green and implying calibration was demonstrated.

use std::path::{Path, PathBuf};

/// One measured construct: what was built, and what it did.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasuredConstruct {
    /// Identifier from the source dataset, so a row can be traced back.
    pub id: String,
    /// The promoter sequence, if the dataset separates it.
    pub promoter: Option<String>,
    /// The ribosome binding site sequence.
    pub rbs: String,
    /// Measured protein output, in the source's own units.
    ///
    /// For Kosuri this is a GFP/RFP fluorescence ratio. Units are not
    /// normalised across datasets, deliberately: a calibration fitted on one
    /// dataset's units does not transfer to another's, and pretending otherwise
    /// by rescaling would hide exactly the distribution shift that matters.
    pub protein: f64,
    /// Measured RNA level, where the dataset reports it.
    pub rna: Option<f64>,
}

/// Errors from dataset loading.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DatasetError {
    /// The dataset is not present at the configured path.
    ///
    /// Carries the path and the documentation pointer, because a user hitting
    /// this needs to know what to do next, not merely that something is absent.
    #[error(
        "no calibration dataset at {path}. This data is not distributed with Integrase; \
         see docs/calibration-data.md for how to obtain it. Until it is present, models \
         report Uncalibrated and offer no intervals."
    )]
    NotPresent {
        /// Where it was looked for.
        path: PathBuf,
    },
    /// A row could not be parsed.
    #[error("row {row} of {path}: {detail}")]
    BadRow {
        /// 1-based row number in the source file.
        row: usize,
        /// The file.
        path: PathBuf,
        /// What was wrong.
        detail: String,
    },
    /// The file was readable but contained no usable rows.
    #[error("{path} contained no usable rows; calibration cannot proceed on an empty dataset")]
    Empty {
        /// The file.
        path: PathBuf,
    },
    /// I/O failure.
    #[error("could not read {path}: {source}")]
    Io {
        /// The file.
        path: PathBuf,
        /// Underlying error.
        source: std::io::Error,
    },
}

/// Where calibration data is expected, unless overridden.
///
/// Gitignored. Nothing here is ever committed.
#[must_use]
pub fn default_data_dir() -> PathBuf {
    std::env::var_os("INTEGRASE_DATA_DIR")
        .map_or_else(|| PathBuf::from("data/calibration"), PathBuf::from)
}

/// Loads measured constructs from a tab- or comma-separated file.
///
/// The expected columns are `id`, `rbs`, `protein`, and optionally `promoter`
/// and `rna`. A header row is required, so that a file with columns in a
/// different order is read correctly rather than silently misinterpreted.
///
/// # Errors
///
/// [`DatasetError`] if the file is absent, unreadable, empty, or malformed.
pub fn load_delimited(path: &Path) -> Result<Vec<MeasuredConstruct>, DatasetError> {
    if !path.exists() {
        return Err(DatasetError::NotPresent {
            path: path.to_path_buf(),
        });
    }
    let text = std::fs::read_to_string(path).map_err(|source| DatasetError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    let delim = if text.lines().next().is_some_and(|l| l.contains('\t')) {
        '\t'
    } else {
        ','
    };

    let mut lines = text.lines().enumerate();
    let Some((_, header_line)) = lines.next() else {
        return Err(DatasetError::Empty {
            path: path.to_path_buf(),
        });
    };
    let header: Vec<&str> = header_line.split(delim).map(str::trim).collect();
    let col = |name: &str| header.iter().position(|h| h.eq_ignore_ascii_case(name));

    let (Some(c_id), Some(c_rbs), Some(c_protein)) = (col("id"), col("rbs"), col("protein")) else {
        return Err(DatasetError::BadRow {
            row: 1,
            path: path.to_path_buf(),
            detail: format!(
                "header must contain id, rbs and protein columns; found [{}]",
                header.join(", ")
            ),
        });
    };
    let c_promoter = col("promoter");
    let c_rna = col("rna");

    let mut out = Vec::new();
    for (i, line) in lines {
        let row = i + 1;
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(delim).map(str::trim).collect();
        let get = |idx: usize| -> Result<&str, DatasetError> {
            f.get(idx).copied().ok_or_else(|| DatasetError::BadRow {
                row,
                path: path.to_path_buf(),
                detail: format!("expected at least {} fields, found {}", idx + 1, f.len()),
            })
        };
        let protein: f64 = get(c_protein)?.parse().map_err(|_| DatasetError::BadRow {
            row,
            path: path.to_path_buf(),
            detail: format!("protein value {:?} is not a number", f.get(c_protein)),
        })?;
        if !protein.is_finite() {
            return Err(DatasetError::BadRow {
                row,
                path: path.to_path_buf(),
                detail: "protein value is not finite".into(),
            });
        }
        out.push(MeasuredConstruct {
            id: get(c_id)?.to_owned(),
            promoter: c_promoter.and_then(|c| f.get(c)).map(|s| (*s).to_owned()),
            rbs: get(c_rbs)?.to_owned(),
            protein,
            rna: c_rna.and_then(|c| f.get(c)).and_then(|s| s.parse().ok()),
        });
    }

    if out.is_empty() {
        return Err(DatasetError::Empty {
            path: path.to_path_buf(),
        });
    }
    Ok(out)
}

/// Splits constructs so that no RBS sequence appears on both sides.
///
/// **This is not a random split, and the difference is the point.**
///
/// Kosuri-style libraries are combinatorial: the same RBS appears in many
/// constructs paired with different promoters. A random split puts near
/// duplicates on both sides, so the model is evaluated on sequences it has
/// effectively already seen. That inflates every metric and makes calibration
/// look better than it is.
///
/// Splitting by component measures what a user actually needs: performance on an
/// RBS the model has never seen. It produces worse numbers, and they are the
/// true ones.
///
/// `hash_seed` makes the assignment deterministic, so a reported figure can be
/// reproduced exactly.
#[must_use]
pub fn split_by_component(
    constructs: &[MeasuredConstruct],
    calibration_fraction: f64,
    hash_seed: u64,
) -> (Vec<MeasuredConstruct>, Vec<MeasuredConstruct>) {
    // Assign whole RBS sequences to one side or the other, never individual rows.
    let bucket = |rbs: &str| -> bool {
        // FNV-1a to absorb the bytes.
        let mut h = hash_seed ^ 0xcbf2_9ce4_8422_2325;
        for b in rbs.as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0000_0100_0000_01B3);
        }
        // Then a splitmix64 finaliser, which is not optional.
        //
        // FNV-1a avalanches into the LOW bits and leaves the high bits poorly
        // distributed for short inputs. Sampling `h >> 11` without this step
        // reads exactly those bad bits, and the first version of this function
        // put 0% of a 400-sequence library into the calibration split as a
        // result. The finaliser spreads entropy across all 64 bits.
        h ^= h >> 30;
        h = h.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        h ^= h >> 27;
        h = h.wrapping_mul(0x94d0_49bb_1331_11eb);
        h ^= h >> 31;

        #[allow(clippy::cast_precision_loss)]
        let u = (h >> 11) as f64 / (1u64 << 53) as f64;
        u < calibration_fraction
    };

    let mut cal = Vec::new();
    let mut test = Vec::new();
    for c in constructs {
        if bucket(&c.rbs) {
            cal.push(c.clone())
        } else {
            test.push(c.clone())
        }
    }
    (cal, test)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// A temp file that removes itself on drop, including when a test panics.
    fn temp_file(contents: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("temp file");
        f.write_all(contents.as_bytes()).expect("write");
        f.flush().expect("flush");
        f
    }

    #[test]
    fn a_missing_dataset_says_what_to_do_about_it() {
        let err = load_delimited(Path::new("/nonexistent/calibration.tsv")).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("docs/calibration-data.md"),
            "the error must point at the documentation; got {msg}"
        );
        assert!(
            msg.contains("Uncalibrated"),
            "the error must say what happens in the meantime; got {msg}"
        );
    }

    #[test]
    fn a_tsv_with_a_header_loads() {
        let f = temp_file(
            "id\trbs\tprotein\tpromoter\n\
             c1\tAGGAGG\t4200.0\tPtac\n\
             c2\tAGGAGA\t1300.0\tPtac\n",
        );
        let rows = load_delimited(f.path()).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, "c1");
        assert_eq!(rows[0].protein, 4200.0);
        assert_eq!(rows[0].promoter.as_deref(), Some("Ptac"));
    }

    /// Columns are located by header name, not position, so a file that orders
    /// them differently is read correctly rather than silently misinterpreted.
    #[test]
    fn column_order_does_not_matter() {
        let f = temp_file("protein\tid\trbs\n999.0\tc9\tAGGAGG\n");
        let rows = load_delimited(f.path()).unwrap();
        assert_eq!(rows[0].id, "c9");
        assert_eq!(rows[0].protein, 999.0);
    }

    #[test]
    fn a_missing_required_column_is_refused_and_names_what_was_found() {
        let f = temp_file("id\tsequence\tvalue\nc1\tAGG\t1.0\n");
        let msg = load_delimited(f.path()).unwrap_err().to_string();
        assert!(
            msg.contains("id, sequence, value"),
            "must list the headers found; got {msg}"
        );
    }

    #[test]
    fn a_non_numeric_measurement_is_refused_with_its_row() {
        // Line 1 is the header, line 2 is c1, line 3 is the bad row. The first
        // version of this test asserted 2, which was the test being wrong
        // rather than the code: 1-based line numbering counts the header.
        let f = temp_file("id\trbs\tprotein\nc1\tAGG\t1.0\nc2\tAGA\tnot_a_number\n");
        match load_delimited(f.path()).unwrap_err() {
            DatasetError::BadRow { row, .. } => {
                assert_eq!(
                    row, 3,
                    "1-based line number in the file, counting the header"
                )
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn a_header_only_file_is_refused_rather_than_silently_empty() {
        let f = temp_file("id\trbs\tprotein\n");
        assert!(matches!(
            load_delimited(f.path()),
            Err(DatasetError::Empty { .. })
        ));
    }

    // --- Component-wise splitting ----------------------------------------

    fn constructs(pairs: &[(&str, &str)]) -> Vec<MeasuredConstruct> {
        pairs
            .iter()
            .enumerate()
            .map(|(i, (rbs, prom))| MeasuredConstruct {
                id: format!("c{i}"),
                promoter: Some((*prom).to_owned()),
                rbs: (*rbs).to_owned(),
                protein: i as f64,
                rna: None,
            })
            .collect()
    }

    /// **The property that makes the split honest.** No RBS may appear on both
    /// sides. A random split would violate this constantly on a combinatorial
    /// library and inflate every metric.
    #[test]
    fn no_rbs_appears_in_both_splits() {
        // Ten RBSs crossed with five promoters: the combinatorial shape.
        let mut pairs = Vec::new();
        for r in 0..10 {
            for p in 0..5 {
                pairs.push((
                    Box::leak(format!("RBS{r:02}").into_boxed_str()) as &str,
                    Box::leak(format!("P{p}").into_boxed_str()) as &str,
                ));
            }
        }
        let all = constructs(&pairs);
        let (cal, test) = split_by_component(&all, 0.5, 42);

        let cal_rbs: std::collections::BTreeSet<_> = cal.iter().map(|c| &c.rbs).collect();
        let test_rbs: std::collections::BTreeSet<_> = test.iter().map(|c| &c.rbs).collect();
        let overlap: Vec<_> = cal_rbs.intersection(&test_rbs).collect();
        assert!(
            overlap.is_empty(),
            "an RBS on both sides means the model is evaluated on sequences it has \
             effectively seen: {overlap:?}"
        );
    }

    #[test]
    fn splitting_loses_no_constructs() {
        let all = constructs(&[("A", "P1"), ("B", "P1"), ("C", "P2"), ("A", "P2")]);
        let (cal, test) = split_by_component(&all, 0.5, 7);
        assert_eq!(cal.len() + test.len(), all.len());
    }

    #[test]
    fn the_split_is_deterministic_so_a_result_can_be_reproduced() {
        let all = constructs(&[("A", "P"), ("B", "P"), ("C", "P"), ("D", "P"), ("E", "P")]);
        let a = split_by_component(&all, 0.5, 99);
        let b = split_by_component(&all, 0.5, 99);
        assert_eq!(a, b, "same seed must give the same split");
    }

    #[test]
    fn a_different_seed_gives_a_different_split() {
        let pairs: Vec<(&str, &str)> = (0..40)
            .map(|i| (Box::leak(format!("R{i:02}").into_boxed_str()) as &str, "P"))
            .collect();
        let all = constructs(&pairs);
        let (a, _) = split_by_component(&all, 0.5, 1);
        let (b, _) = split_by_component(&all, 0.5, 2);
        assert_ne!(
            a.iter().map(|c| &c.rbs).collect::<Vec<_>>(),
            b.iter().map(|c| &c.rbs).collect::<Vec<_>>(),
            "different seeds must produce different assignments, or the seed does nothing"
        );
    }

    #[test]
    fn the_split_fraction_is_roughly_honoured() {
        let pairs: Vec<(&str, &str)> = (0..400)
            .map(|i| (Box::leak(format!("R{i:03}").into_boxed_str()) as &str, "P"))
            .collect();
        let all = constructs(&pairs);
        let (cal, _) = split_by_component(&all, 0.3, 5);
        #[allow(clippy::cast_precision_loss)]
        let frac = cal.len() as f64 / all.len() as f64;
        assert!(
            (frac - 0.3).abs() < 0.08,
            "expected roughly 30%, got {frac:.3}"
        );
    }
}
