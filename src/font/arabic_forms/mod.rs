//! Synthesis of Arabic presentation forms that a font only reaches through GSUB.
//!
//! MapLibre renderers shape Arabic by replacing each letter with its presentation form
//! code point (U+FB50–U+FDFF, U+FE70–U+FEFF) and then look that code point up in the
//! glyph ranges. Many Arabic fonts (Lateef, Scheherazade New, Reem Kufi, …) never map
//! those code points in `cmap`, they reach the joined shapes only through GSUB. Their
//! ranges would lack the joined forms, and labels would render as isolated letters.
//!
//! For every presentation form a font does not map, [`synthesize_arabic_forms`] shapes
//! the letters the form stands for with the font's own GSUB and GPOS (via `rustybuzz`),
//! in a joining context made with ZERO WIDTH JOINER, and records the resulting glyphs.
//!
//! The mark forms U+FE70–U+FE7F are not synthesized. In MapLibre shaping they stand
//! for a mark on a tatweel, so a synthesized glyph would add a visible gap after the
//! letter, and renderers handle a missing mark form better on their own.

mod table;

use rustybuzz::{script, Direction, ShapePlan, UnicodeBuffer};
use std::collections::BTreeMap;
use table::ARABIC_FORMS;
use ttf_parser::{Face, GlyphId};

/// The joining form a presentation form code point stands for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
	/// Not joined on either side.
	Isolated,
	/// Joined to the previous letter only.
	Final,
	/// Joined to the next letter only.
	Initial,
	/// Joined on both sides.
	Medial,
}

/// One glyph of a [`SynthesizedGlyph`], with its position in font units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlyphPart {
	/// The glyph in the font.
	pub glyph_id: GlyphId,
	/// Horizontal offset of the glyph from the origin, in font units.
	pub x: i32,
	/// Vertical offset of the glyph from the baseline, in font units.
	pub y: i32,
}

/// A code point drawn from one or more glyphs of a font, placed as the font's
/// shaping rules place them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SynthesizedGlyph {
	/// The glyphs that make up the shape, from left to right.
	pub parts: Vec<GlyphPart>,
	/// The horizontal advance of the whole shape, in font units.
	pub advance: i32,
}

/// ZERO WIDTH JOINER, used to put a letter in a joining context while shaping.
const ZWJ: char = '\u{200D}';

/// Returns a [`SynthesizedGlyph`] for every Arabic presentation form code point that
/// `face` does not map but can draw from the letters it stands for.
///
/// Fonts without Arabic letters return an empty map without any shaping work.
pub fn synthesize_arabic_forms(face: &Face) -> BTreeMap<u32, SynthesizedGlyph> {
	let has_glyph = |cp: u16| {
		char::from_u32(cp as u32)
			.and_then(|c| face.glyph_index(c))
			.is_some()
	};

	let mut forms = BTreeMap::new();
	let mut shaper: Option<(rustybuzz::Face, ShapePlan)> = None;

	for &(codepoint, form, letters) in ARABIC_FORMS {
		if has_glyph(codepoint) || !letters.iter().all(|&l| l == 0x20 || has_glyph(l)) {
			continue;
		}
		let (shaper_face, plan) = shaper.get_or_insert_with(|| {
			let shaper_face = rustybuzz::Face::from_face(face.clone());
			let plan = ShapePlan::new(
				&shaper_face,
				Direction::RightToLeft,
				Some(script::ARABIC),
				None,
				&[],
			);
			(shaper_face, plan)
		});
		if let Some(glyph) = shape_form(shaper_face, plan, form, letters) {
			forms.insert(codepoint as u32, glyph);
		}
	}
	forms
}

/// Shapes `letters` in the joining context of `form` and returns the glyphs that
/// belong to the letters, or `None` if the font draws nothing for them.
fn shape_form(
	face: &rustybuzz::Face,
	plan: &ShapePlan,
	form: Form,
	letters: &[u16],
) -> Option<SynthesizedGlyph> {
	let joined_before = matches!(form, Form::Final | Form::Medial);
	let joined_after = matches!(form, Form::Initial | Form::Medial);

	// Clusters number the input characters, so the letters are `first..end`.
	let first = joined_before as u32;
	let end = first + letters.len() as u32;

	let mut buffer = UnicodeBuffer::new();
	buffer.set_direction(Direction::RightToLeft);
	buffer.set_script(script::ARABIC);
	if joined_before {
		buffer.add(ZWJ, 0);
	}
	for (cluster, &letter) in (first..).zip(letters) {
		buffer.add(char::from_u32(letter as u32)?, cluster);
	}
	if joined_after {
		buffer.add(ZWJ, end);
	}

	let shaped = rustybuzz::shape_with_plan(face, plan, buffer);

	// Glyphs come in visual order, left to right.
	let mut parts = Vec::new();
	let mut x = 0;
	for (info, pos) in shaped.glyph_infos().iter().zip(shaped.glyph_positions()) {
		if !(first..end).contains(&info.cluster) {
			continue;
		}
		// Skip `.notdef` and glyphs without an outline, like a ZERO WIDTH JOINER that
		// shaping merged into the cluster of the letter before it.
		let glyph_id = GlyphId(info.glyph_id as u16);
		if glyph_id.0 != 0 && face.glyph_bounding_box(glyph_id).is_some() {
			parts.push(GlyphPart {
				glyph_id,
				x: x + pos.x_offset,
				y: pos.y_offset,
			});
		}
		x += pos.x_advance;
	}

	(!parts.is_empty()).then_some(SynthesizedGlyph { parts, advance: x })
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Noto Sans Arabic, subset to U+0020, U+0600–U+06FF and U+200C–U+200D with all layout
	/// features, and renamed to "Arabic Forms Test" so it does not merge with the Noto fonts.
	/// It has every joined glyph, but maps no presentation form code point.
	const NOTO_ARABIC_GSUB_ONLY: &[u8] =
		include_bytes!("../../../testdata/Arabic Forms Test - Regular.ttf");
	const NOTO_ARABIC: &[u8] =
		include_bytes!("../../../testdata/Noto Sans/Noto Sans Arabic - Regular.ttf");
	const FIRA: &[u8] = include_bytes!("../../../testdata/Fira Sans - Regular.ttf");

	#[test]
	fn test_table_is_sorted_and_skips_mark_forms() {
		assert!(ARABIC_FORMS.windows(2).all(|w| w[0].0 < w[1].0));
		assert!(ARABIC_FORMS
			.iter()
			.all(|f| !(0xFE70..0xFE80).contains(&f.0)));
		assert_eq!(ARABIC_FORMS.len(), 717);
	}

	#[test]
	fn test_no_arabic_letters_means_no_forms() {
		let face = Face::parse(FIRA, 0).unwrap();
		assert!(synthesize_arabic_forms(&face).is_empty());
	}

	#[test]
	fn test_mapped_forms_are_kept() {
		let face = Face::parse(NOTO_ARABIC, 0).unwrap();
		let forms = synthesize_arabic_forms(&face);
		assert!(forms
			.keys()
			.all(|&cp| face.glyph_index(char::from_u32(cp).unwrap()).is_none()));
		assert!(!forms.contains_key(&0xFE91));
	}

	#[test]
	fn test_forms_from_gsub() {
		let face = Face::parse(NOTO_ARABIC_GSUB_ONLY, 0).unwrap();
		assert!(face.glyph_index('\u{FE91}').is_none());

		let forms = synthesize_arabic_forms(&face);
		assert!(forms.len() > 300, "{}", forms.len());

		// Every form of BEH is one glyph, and all four differ.
		let beh = [0xFE8F, 0xFE90, 0xFE91, 0xFE92].map(|cp| {
			let glyph = &forms[&cp];
			assert_eq!(glyph.parts.len(), 1, "{cp:04X}");
			assert_eq!((glyph.parts[0].x, glyph.parts[0].y), (0, 0), "{cp:04X}");
			glyph.parts[0].glyph_id
		});
		assert!(beh
			.iter()
			.enumerate()
			.all(|(i, a)| beh[i + 1..].iter().all(|b| a != b)));

		// The isolated form is the letter itself.
		assert_eq!(Some(beh[0]), face.glyph_index('\u{0628}'));

		// LAM WITH ALEF is drawn, as a ligature or as two glyphs.
		assert!(forms.contains_key(&0xFEFB));
		assert!(forms.contains_key(&0xFEFC));
	}
}
