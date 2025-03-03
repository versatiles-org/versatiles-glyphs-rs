fn bitmap_as_strings<F>(bitmap: &[u8], width: usize, func: F) -> Vec<String>
where
	F: Fn(&u8) -> String,
	F: Copy,
{
	bitmap
		.chunks(width)
		.map(|row| row.iter().map(func).collect::<Vec<String>>().join(" "))
		.collect()
}

pub fn bitmap_as_digit_art(bitmap: &[u8], width: usize) -> Vec<String> {
	bitmap_as_strings(bitmap, width, |&x| {
		let v = 100.0 + (x as f32) / 2.56;
		let s = v.to_string();
		String::from(&s[1..3])
	})
}
pub fn bitmap_as_ascii_art(bitmap: &[u8], width: usize) -> Vec<String> {
	bitmap_as_strings(bitmap, width, |&x| {
		String::from(match x {
			0..=60 => " ",
			61..=120 => "░",
			121..=180 => "▒",
			181..=240 => "▓",
			_ => "█",
		})
	})
}
