use super::BUFFER;
use crate::protobuf::PbfGlyph;

#[derive(Debug, Default)]
pub struct SdfGlyph {
	pub x0: i32,
	pub x1: i32,
	pub y0: i32,
	pub y1: i32,

	pub width: u32,
	pub height: u32,

	pub bitmap: Option<Vec<u8>>,
}

impl SdfGlyph {
	pub fn into_pbf_glyph(self, id: u32, advance: u32) -> PbfGlyph {
		PbfGlyph {
			id,
			bitmap: self.bitmap,
			width: self.width - 2 * BUFFER as u32,
			height: self.height - 2 * BUFFER as u32,
			left: self.x0 + BUFFER,
			top: self.y1 - BUFFER,
			advance,
		}
	}
}
