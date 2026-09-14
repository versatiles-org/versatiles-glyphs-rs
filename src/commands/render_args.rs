use crate::{
	render::{Renderer, MAX_SDF_STEP},
	utils::prepare_output_directory,
	writer::Writer,
};
use anyhow::Result;
use std::{io::Write, path};

/// Output and rendering arguments shared by the `merge` and `recurse` subcommands.
#[derive(clap::Args, Debug)]
pub struct RenderArgs {
	/// Output directory for glyphs. Mutually exclusive with `tar`.
	#[arg(long, short = 'o', conflicts_with = "tar")]
	pub output_directory: Option<String>,

	/// Write glyphs as a tar to stdout. Mutually exclusive with `output_directory`.
	#[arg(long, short = 't', conflicts_with = "output_directory")]
	pub tar: bool,

	/// Quantize SDF values to multiples of this step: 1, 2, 4, 8, 16, 32 or 64.
	///
	/// Larger steps make the glyphs compress much better, at the cost of precision:
	/// 1 = 1/32 px (default, full precision), 2 = 1/16 px, 4 = 1/8 px, 8 = 1/4 px, …
	#[arg(long, default_value_t = 1, value_parser = parse_sdf_step)]
	pub sdf_step: u8,

	/// Hidden argument to allow specifying the dummy renderer.
	#[arg(long, hide = true)]
	pub dummy: bool,

	/// Hidden argument to render glyphs in just a single thread.
	#[arg(long, hide = true)]
	pub single_thread: bool,
}

impl Default for RenderArgs {
	fn default() -> Self {
		Self {
			output_directory: None,
			tar: false,
			sdf_step: 1,
			dummy: false,
			single_thread: false,
		}
	}
}

/// Parses and validates the `--sdf-step` value.
fn parse_sdf_step(value: &str) -> Result<u8, String> {
	let step = value
		.parse::<u8>()
		.ok()
		.filter(|step| step.is_power_of_two() && *step <= MAX_SDF_STEP)
		.ok_or_else(|| format!("must be a power of two from 1 to {MAX_SDF_STEP}"))?;
	Ok(step)
}

impl RenderArgs {
	/// Creates the [`Writer`]: a tar written to `stdout`, or a freshly prepared
	/// output directory (default: `output`).
	pub fn get_writer<'a, W: Write + Send + Sync + 'static>(
		&self,
		stdout: &'a mut W,
	) -> Result<Writer<'a>> {
		Ok(if self.tar {
			eprintln!("Rendering glyphs as tar to stdout.");
			Writer::new_tar(stdout)
		} else {
			let out_dir =
				prepare_output_directory(self.output_directory.as_deref().unwrap_or("output"))?;
			eprintln!("Rendering glyphs to directory: {out_dir:?}");
			Writer::new_file(path::absolute(out_dir)?)
		})
	}

	/// Creates the [`Renderer`].
	pub fn get_renderer(&self) -> Result<Renderer> {
		Renderer::new(self.dummy).with_sdf_step(self.sdf_step)
	}

	/// Whether rendering runs in parallel.
	pub fn parallel(&self) -> bool {
		!self.single_thread
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::Parser;

	#[derive(Parser, Debug)]
	struct Cli {
		#[command(flatten)]
		render: RenderArgs,
	}

	#[test]
	fn test_parse_defaults() {
		let cli = Cli::try_parse_from(["test"]).unwrap();
		assert_eq!(cli.render.output_directory, None);
		assert!(!cli.render.tar);
		assert_eq!(cli.render.sdf_step, 1);
		assert!(cli.render.parallel());

		// `Default` matches the CLI defaults.
		let default = RenderArgs::default();
		assert_eq!(default.sdf_step, cli.render.sdf_step);
		assert_eq!(default.output_directory, cli.render.output_directory);
		assert!(default.get_renderer().is_ok());
	}

	#[test]
	fn test_parse_sdf_step() {
		for step in ["1", "2", "4", "8", "16", "32", "64"] {
			let cli = Cli::try_parse_from(["test", "--sdf-step", step]).unwrap();
			assert_eq!(cli.render.sdf_step.to_string(), step);
			assert!(cli.render.get_renderer().is_ok());
		}
		for step in ["0", "3", "128", "256", "-4", "abc", ""] {
			let arg = format!("--sdf-step={step}");
			let err = Cli::try_parse_from(["test", &arg]).unwrap_err();
			assert!(err.to_string().contains("power of two"), "{step}: {err}");
		}
	}

	#[test]
	fn test_parse_flags() {
		let cli = Cli::try_parse_from(["test", "-o", "dir", "--dummy", "--single-thread"]).unwrap();
		assert_eq!(cli.render.output_directory.as_deref(), Some("dir"));
		assert!(cli.render.dummy);
		assert!(!cli.render.parallel());
	}

	#[test]
	fn test_tar_conflicts_with_output_directory() {
		assert!(Cli::try_parse_from(["test", "-t", "-o", "dir"]).is_err());
	}
}
