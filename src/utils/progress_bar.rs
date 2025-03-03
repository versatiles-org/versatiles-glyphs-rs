use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

pub fn get_progress_bar(len: u64) -> ProgressBar {
	ProgressBar::with_draw_target(Some(len), ProgressDrawTarget::stderr())
		.with_position(0)
		.with_style(
			ProgressStyle::with_template("{wide_bar} {pos:>8}/{len:8} {eta_precise:8}").unwrap(),
		)
}
