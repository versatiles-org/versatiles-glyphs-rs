mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Command-line interface for VersaTiles
#[derive(Parser, Debug)]
#[command(
	author,
	version,
	about,
	long_about,
	propagate_version = true, // Enable version flag for subcommands
	//disable_help_subcommand = true, // Disable help subcommand
)]
struct Cli {
	#[command(subcommand)]
	command: Commands, // Set subcommands
}

#[derive(Subcommand, Debug)]
enum Commands {
	Merge(commands::merge::Subcommand),
	Recurse(commands::recurse::Subcommand),
}

// ---------------------------------------------------------
// Demo entry point
// ---------------------------------------------------------

fn main() -> Result<()> {
	let cli = Cli::parse();

	match &cli.command {
		Commands::Merge(arguments) => commands::merge::run(arguments),
		Commands::Recurse(arguments) => commands::recurse::run(arguments),
	}
}
