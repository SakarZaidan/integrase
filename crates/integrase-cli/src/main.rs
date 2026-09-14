//! Integrase command-line interface.

use clap::{Parser, Subcommand};

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
    /// Present as a command rather than buried in documentation because a user
    /// who is about to trust an output should be able to ask what it means
    /// without leaving the terminal.
    Scope,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Scope => {
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
            println!(
                "sequence-only model and no interval computed from sequence accounts for them."
            );
            println!();
            println!("No output of this system substitutes for experimental validation.");
        }
    }
    Ok(())
}
