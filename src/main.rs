//! Command-line interface for VersaTiles glyph generation.
//!
//! This binary provides subcommands for merging or recursively scanning
//! font files into a directory or tar archive of glyphs.

mod commands;
mod font;
mod geometry;
mod protobuf;
mod render;
mod utils;
mod writer;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Top-level CLI options.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about, propagate_version = true)]
struct Cli {
	/// Subcommand to run.
	#[command(subcommand)]
	command: Commands,
}

/// Enumeration of subcommands.
#[derive(Subcommand, Debug)]
enum Commands {
	/// Merge subcommand.
	Merge(commands::merge::Subcommand),
	/// Recurse subcommand.
	Recurse(commands::recurse::Subcommand),
	/// Merge subcommand.
	Debug(commands::debug::Subcommand),
}

fn main() -> Result<()> {
	let cli = Cli::parse();
	match &cli.command {
		Commands::Debug(args) => commands::debug::run(args, &mut std::io::stdout())?,
		Commands::Merge(args) => commands::merge::run(args, &mut std::io::stdout())?,
		Commands::Recurse(args) => commands::recurse::run(args, &mut std::io::stdout())?,
	};
	Ok(())
}
