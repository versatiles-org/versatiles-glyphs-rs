mod commands;
mod libraries;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Command-line interface for VersaTiles
#[derive(Parser, Debug)]
#[command(
	author, // Set the author
	version, // Set the version
	about, // Set a short description
	long_about = None, // Disable long description
	propagate_version = false, // Enable version flag for subcommands
	disable_help_subcommand = true, // Disable help subcommand
)]
struct Cli {
	#[command(subcommand)]
	command: Commands, // Set subcommands
}

#[derive(Subcommand, Debug)]
enum Commands {
	Convert(commands::convert::Subcommand),
	
	//Debug(commands::convert::Subcommand),
}

// ---------------------------------------------------------
// Demo entry point
// ---------------------------------------------------------

fn main() -> Result<()> {
	let cli = Cli::parse();

	match &cli.command {
		Commands::Convert(arguments) => commands::convert::run(arguments),
		//Commands::Debug(arguments) => commands::debug::run(arguments),
	}
}
