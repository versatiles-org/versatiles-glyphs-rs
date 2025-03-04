#[derive(serde::Serialize)]
pub struct FontFace {
	pub id: String,
	pub style: String,
	pub weight: u16,
	pub width: String,
}

#[derive(serde::Serialize)]
pub struct FontFamily {
	pub name: String,
	pub faces: Vec<FontFace>,
}

impl FontFamily {
	pub fn new(name: String) -> FontFamily {
		FontFamily {
			name,
			faces: Vec::new(),
		}
	}

	pub fn add_font(&mut self, id: String, style: String, weight: u16, width: String) {
		self.faces.push(FontFace {
			id,
			style,
			weight,
			width,
		});
	}
}
