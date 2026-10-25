use std::{fs, path::PathBuf, process::ExitCode};

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// The old diagram
    old: PathBuf,
    /// The new diagram
    new: PathBuf,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    for path in [&cli.old, &cli.new] {
        // Read, not just open: opening a directory succeeds on Unix, reading it fails.
        if let Err(err) = fs::read(path) {
            eprintln!("runediff: {}: {err}", path.display());
            return ExitCode::from(2);
        }
    }
    eprintln!("runediff: not implemented yet");
    ExitCode::from(2)
}
