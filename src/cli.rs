//! Command-line entry point for the current application shell.

use clap::{CommandFactory, Parser, Subcommand};
use std::io;

/// Parsed command-line arguments for Head Huntress.
#[derive(Debug, Parser)]
#[command(
    name = "head-huntress",
    version,
    about = "A configurable job-hunting assistant"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

/// Commands exposed by the initial CLI shell.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Discover and ingest jobs from configured sources.
    Sync,
    /// Evaluate jobs against configured hard and semantic rules.
    Evaluate,
    /// Generate a tailored resume plan for a job.
    Resume,
    /// Review jobs awaiting a human decision.
    Review,
    /// Start the future API or GUI application shell.
    Serve,
}

/// Parses the CLI and dispatches the current placeholder commands.
///
/// The commands intentionally do not execute application workflows yet. They
/// establish the interface that can later be shared by a terminal UI, an HTTP
/// API, or a desktop GUI.
pub fn run() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(command) => {
            println!(
                "The `{}` command is not implemented yet.",
                command_name(&command)
            );
        }
        None => {
            let mut command = Cli::command();
            command.print_help()?;
            println!();
        }
    }

    Ok(())
}

fn command_name(command: &Command) -> &'static str {
    match command {
        Command::Sync => "sync",
        Command::Evaluate => "evaluate",
        Command::Resume => "resume",
        Command::Review => "review",
        Command::Serve => "serve",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_names_match_the_cli_commands() {
        assert_eq!(command_name(&Command::Sync), "sync");
        assert_eq!(command_name(&Command::Evaluate), "evaluate");
        assert_eq!(command_name(&Command::Resume), "resume");
        assert_eq!(command_name(&Command::Review), "review");
        assert_eq!(command_name(&Command::Serve), "serve");
    }
}
