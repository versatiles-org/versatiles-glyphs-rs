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
