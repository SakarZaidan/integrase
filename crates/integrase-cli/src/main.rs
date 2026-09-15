//! Integrase command-line interface.
//!
//! Output is designed to be read by a person and piped by a script. Predictions
//! print their interval and domain verdict inline rather than behind a flag,
//! because uncertainty that needs asking for is uncertainty that gets ignored.

use clap::{Parser, Subcommand, ValueEnum};
use integrase_core::{COMMON_ENZYMES, find_pattern, find_restriction_sites};
use integrase_screen::{Gateway, NullBackend, ScreeningGateway};
use integrase_sim::{DomainVerdict, RbsInput, RbsThermodynamicModel};
use std::path::PathBuf;

/// Genetic design with the error bars attached.
#[derive(Debug, Parser)]
#[command(name = "integrase", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the scope limits this tool operates under.
    ///
    /// A command rather than a documentation page, because a user about to trust
    /// an output should be able to ask what it means without leaving the
    /// terminal.
    Scope,

    /// Read an SBOL file and summarise what came through.
    Import {
        /// Path to an SBOL 3 Turtle file.
        path: PathBuf,
        /// Show every conversion note, not just the count.
        #[arg(long)]
        notes: bool,
    },

    /// Predict translation initiation rate for a ribosome binding site.
    Predict {
        /// The 5' untranslated region immediately upstream of the start codon.
        utr: String,
        /// Start codon, exactly three bases.
        #[arg(long, default_value = "ATG")]
        start_codon: String,
        /// Host organism.
        #[arg(long, default_value = "Escherichia coli")]
        host: String,
    },

    /// Search a sequence for a motif or for restriction sites.
    Search {
        /// The sequence to search.
        sequence: String,
        /// An IUPAC motif. Omit to scan for common restriction sites instead.
        #[arg(long)]
        motif: Option<String>,
    },

    /// Write a design toward physical synthesis.
    ///
    /// Requires a configured screening backend. With none, this refuses, which
    /// is the intended behaviour rather than a limitation: passing everything
    /// when unconfigured would let a user believe they had been screened.
    Export {
        /// Path to an SBOL 3 Turtle file to export.
        path: PathBuf,
        /// Output format.
        #[arg(long, value_enum, default_value_t = Format::Fasta)]
        format: Format,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    /// FASTA, which most synthesis vendors accept.
    Fasta,
}

fn main() -> std::process::ExitCode {
    match run(Cli::parse()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Scope => print_scope(),
        Command::Import { path, notes } => import(&path, notes)?,
        Command::Predict {
            utr,
            start_codon,
            host,
        } => predict(&utr, &start_codon, &host)?,
        Command::Search { sequence, motif } => search(&sequence, motif.as_deref())?,
        Command::Export { path, format } => export(&path, format)?,
    }
    Ok(())
}

fn print_scope() {
    println!("Integrase operates on sequence data.");
    println!();
    println!("It does NOT:");
    println!("  - edit genomes");
    println!("  - interact with living organisms");
    println!("  - order or produce physical DNA");
    println!("  - predict what will happen in a cell");
    println!();
    println!("Predictions are model outputs, not measurements. Their intervals describe");
    println!("uncertainty about the model's prediction, not about the biological outcome.");
    println!("Genomic context, host physiology, and growth conditions are invisible to a");
    println!("sequence-only model and no interval computed from sequence accounts for them.");
    println!();
    println!("No output of this system substitutes for experimental validation.");
}

fn import(path: &std::path::Path, show_notes: bool) -> anyhow::Result<()> {
    let text = std::fs::read_to_string(path)?;
    let result = integrase_sbol::from_turtle(&text)?;

    println!(
        "{} design(s) imported from {}",
        result.designs.len(),
        path.display()
    );
    for d in &result.designs {
        println!(
            "  {}  {} bp  {} feature(s)",
            d.id.0,
            d.sequence.len(),
            d.features.len()
        );
    }

    // Conversion notes are surfaced, not discarded. What the importer could not
    // carry is the part a user most needs to know.
    if result.report.is_silent() {
        println!("\nnothing was dropped or altered in conversion");
    } else {
        let n = result.report.notes.len();
        println!("\n{n} conversion note(s):");
        if show_notes {
            for note in &result.report.notes {
                println!("  - {note}");
            }
        } else {
            for note in result.report.notes.iter().take(3) {
                println!("  - {note}");
            }
            if n > 3 {
                println!("  ... and {} more; pass --notes to see all", n - 3);
            }
        }
    }
    Ok(())
}

fn predict(utr: &str, start_codon: &str, host: &str) -> anyhow::Result<()> {
    let codon: [u8; 3] = start_codon
        .as_bytes()
        .try_into()
        .map_err(|_| anyhow::anyhow!("start codon must be exactly 3 bases, got {start_codon:?}"))?;

    let model = RbsThermodynamicModel::new();
    let input = RbsInput {
        utr: utr.as_bytes().to_vec(),
        start_codon: codon,
        host: host.into(),
    };
    let p = model.predict(&input)?;

    println!("model     {}", p.model().0);
    println!("estimate  {:.4}", p.value());

    match p.interval() {
        Some(i) => println!(
            "interval  [{:.4}, {:.4}]   ({:.0}% coverage)",
            i.lower,
            i.upper,
            p.nominal_coverage() * 100.0
        ),
        // Absence is reported with its reason, not left blank. A blank field
        // reads as missing data; this is a deliberate statement.
        None => println!("interval  none: this model has no calibration evidence"),
    }

    println!("domain    {}", describe_domain(p.domain()));

    if p.domain().coverage_is_unreliable() {
        println!();
        println!("This estimate has no usable coverage guarantee. Treat it as a plausible");
        println!("magnitude, not as a prediction with a confidence level.");
    }
    Ok(())
}

fn describe_domain(v: &DomainVerdict) -> String {
    match v {
        DomainVerdict::InDistribution { similarity } => {
            format!("in distribution (similarity {similarity:.2})")
        }
        DomainVerdict::Marginal { similarity, reason } => {
            format!("marginal (similarity {similarity:.2}): {reason}")
        }
        DomainVerdict::OutOfDistribution { reason } => {
            format!("OUT OF DISTRIBUTION: {reason}")
        }
        DomainVerdict::Uncalibrated => {
            "uncalibrated: this model has not demonstrated coverage on held-out data".into()
        }
        _ => "unknown".into(),
    }
}

fn search(sequence: &str, motif: Option<&str>) -> anyhow::Result<()> {
    let seq = integrase_core::Sequence::new(
        sequence.as_bytes().to_vec(),
        integrase_core::Alphabet::DnaIupac,
    )?;
    let view = seq.view();

    match motif {
        Some(m) => {
            let hits = find_pattern(&view, m.as_bytes())?;
            println!("{} match(es) for {m}", hits.len());
            for h in hits {
                println!("  {}..{}  {:?}", h.start, h.end, h.strand);
            }
        }
        None => {
            let sites = find_restriction_sites(&view, COMMON_ENZYMES)?;
            println!("{} restriction site(s)", sites.len());
            for s in sites {
                println!(
                    "  {:<8} {}..{}  {:?}",
                    s.enzyme, s.location.start, s.location.end, s.location.strand
                );
            }
        }
    }
    Ok(())
}

fn export(path: &std::path::Path, format: Format) -> anyhow::Result<()> {
    let text = std::fs::read_to_string(path)?;
    let result = integrase_sbol::from_turtle(&text)?;
    let design =
        result.designs.into_iter().next().ok_or_else(|| {
            anyhow::anyhow!("{} contains no design with a sequence", path.display())
        })?;

    // No backend is configured, so this refuses. That is the designed
    // behaviour: the alternative, passing everything when unconfigured, would
    // let a user believe they had been screened when they had not.
    let gateway = Gateway::new(NullBackend);
    match gateway.screen(design) {
        Ok(cleared) => {
            let out = match format {
                Format::Fasta => integrase_sbol::export_for_synthesis(
                    &cleared,
                    integrase_sbol::VendorFormat::Fasta,
                )?,
            };
            print!("{out}");
            Ok(())
        }
        Err(refusal) => {
            eprintln!("{}", refusal.explain());
            Err(anyhow::anyhow!("synthesis-bound export refused"))
        }
    }
}
