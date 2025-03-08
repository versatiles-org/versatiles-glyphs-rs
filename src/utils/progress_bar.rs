use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

/// Creates and returns an [`indicatif::ProgressBar`] preconfigured for
/// console output. This function sets a default style and automatically
/// hides progress in test contexts.
///
/// # Arguments
///
/// * `len` - The total length or count for the progress bar.
///
/// # Returns
///
/// A [`ProgressBar`] that:
/// - Draws to stderr by default (hidden during tests).
/// - Uses a template displaying a wide progress bar, the current position,
///   total length, and a precise ETA.
///
/// # Example
///
/// ```
/// use versatiles_glyphs::utils::get_progress_bar;
///
/// let pb = get_progress_bar(100);
/// pb.inc(10);
/// assert_eq!(pb.position(), 10);
/// ```
pub fn get_progress_bar(len: u64) -> ProgressBar {
	#[cfg(not(test))]
	let target: ProgressDrawTarget = ProgressDrawTarget::stderr();
	#[cfg(test)]
	let target: ProgressDrawTarget = ProgressDrawTarget::hidden();

	ProgressBar::with_draw_target(Some(len), target)
		.with_position(0)
		.with_style(
			ProgressStyle::with_template("{wide_bar} {pos:>8}/{len:8} {eta_precise:8}").unwrap(),
		)
}
