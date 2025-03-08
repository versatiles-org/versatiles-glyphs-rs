/// Converts a grayscale bitmap into rows of two-digit numeric strings,
/// each representing the pixel's approximate grayscale value.
///
/// Each pixel (`u8`) is first scaled by adding `100.0` and dividing by `2.56`,
/// then truncated to a two-character substring. For instance, a value of `128`
/// might become `"50"` in the output string.
///
/// # Arguments
/// * `bitmap` - A slice of 8-bit grayscale pixel data.
/// * `width` - The number of pixels per row.
///
/// # Returns
/// A vector of strings, where each string represents one row of the image
/// (with each pixel replaced by two digits and separated by spaces).
///
/// # Example
/// ```
/// let bitmap = vec![0, 128, 255, 64];
/// let rows = bitmap_as_digit_art(&bitmap, 2);
/// assert_eq!(
///     rows,
///     vec![
///         "00 50",
///         "12 25",
///     ]
/// );
/// ```
pub fn bitmap_as_digit_art(bitmap: &[u8], width: usize) -> Vec<String> {
	bitmap
		.chunks(width)
		.map(|row| {
			row.iter()
				.map(|&x| {
					let v = 100.0 + (x as f32) / 2.56;
					let s = v.to_string();
					String::from(&s[1..3])
				})
				.collect::<Vec<String>>()
				.join(" ")
		})
		.collect()
}

/// Converts a grayscale bitmap into rows of ASCII "art," using various Unicode
/// characters to represent ranges of pixel intensity.
///
/// The conversion uses these thresholds (inclusively):
///
/// - `0..=60` -> `"  "` (2 spaces)  
/// - `61..=120` -> `"░░"`  
/// - `121..=180` -> `"▒▒"`  
/// - `181..=240` -> `"▓▓"`  
/// - `241..=255` -> `"█"`  
///
/// # Arguments
/// * `bitmap` - A slice of 8-bit grayscale pixel data.
/// * `width` - The number of pixels per row.
///
/// # Returns
/// A vector of strings, where each string represents one row of the image
/// as a sequence of 2-character intensity symbols.
///
/// # Example
/// ```
/// let bitmap = vec![0, 64, 128, 192, 255];
/// let rows = bitmap_as_ascii_art(&bitmap, 5);
/// assert_eq!(
///     rows,
///     vec![
///         "  ░░▒▒▓▓█"
///     ]
/// );
/// ```
pub fn bitmap_as_ascii_art(bitmap: &[u8], width: usize) -> Vec<String> {
	bitmap
		.chunks(width)
		.map(|row| {
			row.iter()
				.map(|&x| {
					String::from(match x {
						0..=60 => "  ",
						61..=120 => "░░",
						121..=180 => "▒▒",
						181..=240 => "▓▓",
						_ => "█",
					})
				})
				.collect::<Vec<String>>()
				.join("")
		})
		.collect()
}
