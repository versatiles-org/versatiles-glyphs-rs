use crate::{render::Renderer, utils::prepare_output_directory, writer::Writer};
use anyhow::Result;
use std::{io::Write, path};

/// Output and rendering arguments shared by the `merge` and `recurse` subcommands.
#[derive(clap::Args, Debug, Default)]
pub struct RenderArgs {
	/// Output directory for glyphs. Mutually exclusive with `tar`.
	#[arg(long, short = 'o', conflicts_with = "tar")]
	pub output_directory: Option<String>,

	/// Write glyphs as a tar to stdout. Mutually exclusive with `output_directory`.
	#[arg(long, short = 't', conflicts_with = "output_directory")]
	pub tar: bool,

	/// Hidden argument to allow specifying the dummy renderer.
	#[arg(long, hide = true)]
	pub dummy: bool,

	/// Hidden argument to render glyphs in just a single thread.
	#[arg(long, hide = true)]
	pub single_thread: bool,
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
	pub fn get_renderer(&self) -> Renderer {
		Renderer::new(self.dummy)
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
		assert!(cli.render.parallel());
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
