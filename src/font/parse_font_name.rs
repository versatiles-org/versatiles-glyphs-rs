//! Utilities for parsing font family, style, weight, and width from raw name strings.

/// Attempts to parse a font's family, style, weight, and width from a given family name and
/// postscript name. Returns a tuple `(family, style, weight, width)`.
///
/// # Arguments
///
/// * `family` - The raw font family name, which may contain extra tokens (e.g. "Open Sans SemiCondensed Light").
/// * `ps_name` - A PostScript-style font name (e.g. "OpenSansSemiCondensed-LightItalic"),
///   which often contains suffixes specifying style or weight (e.g. `-Italic`, `-Bold`, etc.).
///
/// # Returns
///
/// A tuple of:
/// 1. **family**: Cleaned-up family name with extraneous tokens removed (e.g. "Open Sans").
/// 2. **style**: Derived style, typically `"normal"` or `"italic"`.
/// 3. **weight**: A `u16` representing the detected font weight (e.g. 400 for Regular).
/// 4. **width**: Derived width descriptor (e.g. `"normal"`, `"condensed"`, etc.).
///
/// # Examples
///
/// ```
/// use versatiles_glyphs::font::parse_font_name::parse_font_name;
///
/// // Typical usage with a family name and a postscript name
/// let (family, style, weight, width) = parse_font_name(
///     "Open Sans SemiCondensed Light".to_string(),
///     "OpenSansSemiCondensed-LightItalic".to_string()
/// );
/// assert_eq!(family, "Open Sans");
/// assert_eq!(style, "italic");
/// assert_eq!(weight, 300);
/// assert_eq!(width, "semi-condensed");
/// ```
pub fn parse_font_name(family: String, ps_name: String) -> (String, String, u16, String) {
	// Default values
	let mut style = "normal".to_string();
	let mut weight = 400u16;
	let mut width = "normal".to_string();

	// Identify the substring after the last '-' in the PostScript name, if any.
	let suffix = if let Some(pos) = ps_name.rfind('-') {
		&ps_name[pos + 1..]
	} else {
		&ps_name
	};
	let lower_suffix = suffix.to_lowercase();

	// Detect italic style from the PostScript suffix.
	if lower_suffix.contains("italic") {
		style = "italic".to_string();
	}

	// Detect weight from the PostScript suffix.
	let ps_weight = find_weight(&lower_suffix);
	if ps_weight != 400 {
		weight = ps_weight;
	}

	// Parse tokens from the family string to remove known width/weight references.
	let tokens = family.split_whitespace().collect::<Vec<&str>>();
	let mut final_family_tokens = Vec::new();
	let mut i = 0;

	while i < tokens.len() {
		let t = tokens[i].to_lowercase();

		// Check for multi-word widths like "Extra Condensed".
		if i + 1 < tokens.len() && t == "extra" && tokens[i + 1].to_lowercase() == "condensed" {
			width = "extra-condensed".to_string();
			i += 2;
			continue;
		}

		// Check for single-token widths.
		if t == "semicondensed" || t == "semi-condensed" {
			width = "semi-condensed".to_string();
			i += 1;
			continue;
		}
		if t == "condensed" {
			width = "condensed".to_string();
			i += 1;
			continue;
		}

		// Skip language-specific tokens (Arabic, JP, etc.).
		if t == "arabic"
			|| t == "armenian"
			|| t == "balinese"
			|| t == "bengali"
			|| t == "devanagari"
			|| t == "ethiopic"
			|| t == "georgian"
			|| t == "gujarati"
			|| t == "gurmukhi"
			|| t == "hebrew"
			|| t == "jp"
			|| t == "javanese"
			|| t == "kr"
			|| t == "kannada"
			|| t == "khmer"
			|| t == "lao"
			|| t == "myanmar"
			|| t == "oriya"
			|| t == "sc"
			|| t == "sinhala"
			|| t == "tamil"
			|| t == "thai"
		{
			i += 1;
			continue;
		}

		// Check if this token corresponds to a known weight.
		let maybe_w = find_weight(&t);
		if maybe_w != 400 {
			// Use this weight only if the PostScript suffix didn't override it.
			if ps_weight == 400 {
				weight = maybe_w;
			}
			i += 1;
			continue;
		}

		// Otherwise, consider this token part of the family.
		final_family_tokens.push(tokens[i]);
		i += 1;
	}

	let final_family = final_family_tokens.join(" ");
	(final_family, style, weight, width)
}

/// Attempts to detect a font weight from the given lowercased token.
/// Returns 400 if no known keyword matches.
///
/// # Examples
///
/// ```
/// use versatiles_glyphs::font::parse_font_name::find_weight;
/// assert_eq!(find_weight("LightItalic"), 300);
/// assert_eq!(find_weight("SemiBold"), 600);
/// assert_eq!(find_weight("Unknown"), 400); // default
/// ```
fn find_weight(s: &str) -> u16 {
	// Check keywords in order from most-specific to least-specific.
	if s.contains("hairline") || s.contains("thin") {
		100
	} else if s.contains("extralight") || s.contains("ultralight") {
		200
	} else if s.contains("light") {
		300
	} else if s.contains("regular") || s.contains("normal") || s.contains("book") {
		400
	} else if s.contains("medium") {
		500
	} else if s.contains("demibold") || s.contains("semibold") {
		600
	} else if s.contains("bold") {
		// Cover "bold", "extrabold", "ultrabold".
		if s.contains("extra") || s.contains("ultra") {
			800
		} else {
			700
		}
	} else if s.contains("black") || s.contains("heavy") {
		900
	} else {
		// Default to 400 if no known keyword is found.
		400
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_font_name() {
		// Each test entry is: family;postscript;expected_family;expected_style;expected_weight;expected_width
		let samples = vec![
		"Open Sans SemiCondensed ExtraBold;OpenSansSemiCondensed-ExtraBold;Open Sans;normal;800;semi-condensed",
		"Open Sans SemiCondensed Light;OpenSansSemiCondensed-LightItalic;Open Sans;italic;300;semi-condensed",
		"Open Sans SemiCondensed SemiBold;OpenSansSemiCondensed-SemiBold;Open Sans;normal;600;semi-condensed",
		"Open Sans SemiCondensed;OpenSansSemiCondensed-Italic;Open Sans;italic;400;semi-condensed",
		"Open Sans SemiCondensed Medium;OpenSansSemiCondensed-MediumItalic;Open Sans;italic;500;semi-condensed",
		"Open Sans SemiCondensed ExtraBold;OpenSansSemiCondensed-ExtraBoldItalic;Open Sans;italic;800;semi-condensed",
		"Open Sans SemiCondensed;OpenSansSemiCondensed-Bold;Open Sans;normal;700;semi-condensed",
		"Open Sans SemiCondensed;OpenSansSemiCondensed-BoldItalic;Open Sans;italic;700;semi-condensed",
		"Open Sans SemiCondensed Medium;OpenSansSemiCondensed-Medium;Open Sans;normal;500;semi-condensed",
		"Open Sans SemiCondensed SemiBold;OpenSansSemiCondensed-SemiBoldItalic;Open Sans;italic;600;semi-condensed",
		"Open Sans SemiCondensed;OpenSansSemiCondensed-Regular;Open Sans;normal;400;semi-condensed",
		"Open Sans SemiCondensed Light;OpenSansSemiCondensed-Light;Open Sans;normal;300;semi-condensed",
		"Open Sans;OpenSans-BoldItalic;Open Sans;italic;700;normal",
		"Open Sans Medium;OpenSans-Medium;Open Sans;normal;500;normal",
		"Open Sans Medium;OpenSans-MediumItalic;Open Sans;italic;500;normal",
		"Open Sans ExtraBold;OpenSans-ExtraBoldItalic;Open Sans;italic;800;normal",
		"Open Sans SemiBold;OpenSans-SemiBold;Open Sans;normal;600;normal",
		"Open Sans;OpenSans-Bold;Open Sans;normal;700;normal",
		"Open Sans ExtraBold;OpenSans-ExtraBold;Open Sans;normal;800;normal",
		"Open Sans Light;OpenSans-LightItalic;Open Sans;italic;300;normal",
		"Open Sans;OpenSans-Italic;Open Sans;italic;400;normal",
		"Open Sans Light;OpenSans-Light;Open Sans;normal;300;normal",
		"Open Sans SemiBold;OpenSans-SemiBoldItalic;Open Sans;italic;600;normal",
		"Open Sans;OpenSans-Regular;Open Sans;normal;400;normal",
		"Libre Baskerville;LibreBaskerville-Regular;Libre Baskerville;normal;400;normal",
		"Libre Baskerville;LibreBaskerville-Bold;Libre Baskerville;normal;700;normal",
		"Libre Baskerville;LibreBaskerville-Italic;Libre Baskerville;italic;400;normal",
		"Noto Sans;NotoSans-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Arabic;NotoSansArabic-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Armenian;NotoSansArmenian-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Balinese;NotoSansBalinese-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Bengali;NotoSansBengali-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Devanagari;NotoSansDevanagari-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Ethiopic;NotoSansEthiopic-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Georgian;NotoSansGeorgian-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Gujarati;NotoSansGujarati-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Gurmukhi;NotoSansGurmukhi-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Hebrew;NotoSansHebrew-Regular;Noto Sans;normal;400;normal",
		"Noto Sans JP;NotoSansJP-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Javanese;NotoSansJavanese-Regular;Noto Sans;normal;400;normal",
		"Noto Sans KR;NotoSansKR-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Kannada;NotoSansKannada-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Khmer;NotoSansKhmer-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Lao;NotoSansLao-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Myanmar;NotoSansMyanmar-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Oriya;NotoSansOriya-Regular;Noto Sans;normal;400;normal",
		"Noto Sans SC;NotoSansSC-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Sinhala;NotoSansSinhala-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Tamil;NotoSansTamil-Regular;Noto Sans;normal;400;normal",
		"Noto Sans Thai;NotoSansThai-Regular;Noto Sans;normal;400;normal",
		"Noto Sans;NotoSans-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Arabic;NotoSansArabic-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Armenian;NotoSansArmenian-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Balinese;NotoSansBalinese-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Bengali;NotoSansBengali-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Devanagari;NotoSansDevanagari-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Ethiopic;NotoSansEthiopic-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Georgian;NotoSansGeorgian-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Gujarati;NotoSansGujarati-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Gurmukhi;NotoSansGurmukhi-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Hebrew;NotoSansHebrew-Bold;Noto Sans;normal;700;normal",
		"Noto Sans JP;NotoSansJP-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Javanese;NotoSansJavanese-Bold;Noto Sans;normal;700;normal",
		"Noto Sans KR;NotoSansKR-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Kannada;NotoSansKannada-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Khmer;NotoSansKhmer-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Lao;NotoSansLao-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Myanmar;NotoSansMyanmar-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Oriya;NotoSansOriya-Bold;Noto Sans;normal;700;normal",
		"Noto Sans SC;NotoSansSC-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Sinhala;NotoSansSinhala-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Tamil;NotoSansTamil-Bold;Noto Sans;normal;700;normal",
		"Noto Sans Thai;NotoSansThai-Bold;Noto Sans;normal;700;normal",
		"Lato;Lato-Regular;Lato;normal;400;normal",
		"Lato Light;Lato-LightItalic;Lato;italic;300;normal",
		"Lato;Lato-Italic;Lato;italic;400;normal",
		"Lato;Lato-Bold;Lato;normal;700;normal",
		"Lato Hairline;Lato-Hairline;Lato;normal;100;normal",
		"Lato Light;Lato-Light;Lato;normal;300;normal",
		"Lato Black;Lato-BlackItalic;Lato;italic;900;normal",
		"Lato Black;Lato-Black;Lato;normal;900;normal",
		"Lato Hairline;Lato-HairlineItalic;Lato;italic;100;normal",
		"Lato;Lato-BoldItalic;Lato;italic;700;normal",
		"Source Sans 3 Black;SourceSans3-BlackItalic;Source Sans 3;italic;900;normal",
		"Source Sans 3 Black;SourceSans3-Black;Source Sans 3;normal;900;normal",
		"Source Sans 3 Medium;SourceSans3-Medium;Source Sans 3;normal;500;normal",
		"Source Sans 3;SourceSans3-Regular;Source Sans 3;normal;400;normal",
		"Source Sans 3 ExtraLight;SourceSans3-ExtraLightItalic;Source Sans 3;italic;200;normal",
		"Source Sans 3;SourceSans3-BoldItalic;Source Sans 3;italic;700;normal",
		"Source Sans 3 ExtraLight;SourceSans3-ExtraLight;Source Sans 3;normal;200;normal",
		"Source Sans 3 SemiBold;SourceSans3-SemiBold;Source Sans 3;normal;600;normal",
		"Source Sans 3 ExtraBold;SourceSans3-ExtraBoldItalic;Source Sans 3;italic;800;normal",
		"Source Sans 3 Light;SourceSans3-LightItalic;Source Sans 3;italic;300;normal",
		"Source Sans 3 ExtraBold;SourceSans3-ExtraBold;Source Sans 3;normal;800;normal",
		"Source Sans 3;SourceSans3-Bold;Source Sans 3;normal;700;normal",
		"Source Sans 3;SourceSans3-Italic;Source Sans 3;italic;400;normal",
		"Source Sans 3 SemiBold;SourceSans3-SemiBoldItalic;Source Sans 3;italic;600;normal",
		"Source Sans 3 Medium;SourceSans3-MediumItalic;Source Sans 3;italic;500;normal",
		"Source Sans 3 Light;SourceSans3-Light;Source Sans 3;normal;300;normal",
		"Fira Sans Extra Condensed Medium;FiraSansExtraCondensed-Medium;Fira Sans;normal;500;extra-condensed",
		"Fira Sans Extra Condensed SemiBold;FiraSansExtraCondensed-SemiBold;Fira Sans;normal;600;extra-condensed",
		"Fira Sans Extra Condensed ExtraLight;FiraSansExtraCondensed-ExtraLightItalic;Fira Sans;italic;200;extra-condensed",
		"Fira Sans Extra Condensed Light;FiraSansExtraCondensed-Light;Fira Sans;normal;300;extra-condensed",
		"Fira Sans Extra Condensed Black;FiraSansExtraCondensed-BlackItalic;Fira Sans;italic;900;extra-condensed",
		"Fira Sans Extra Condensed Thin;FiraSansExtraCondensed-Thin;Fira Sans;normal;100;extra-condensed",
		"Fira Sans Extra Condensed SemiBold;FiraSansExtraCondensed-SemiBoldItalic;Fira Sans;italic;600;extra-condensed",
		"Fira Sans Extra Condensed Black;FiraSansExtraCondensed-Black;Fira Sans;normal;900;extra-condensed",
		"Fira Sans Extra Condensed Thin;FiraSansExtraCondensed-ThinItalic;Fira Sans;italic;100;extra-condensed",
		"Fira Sans Extra Condensed;FiraSansExtraCondensed-Bold;Fira Sans;normal;700;extra-condensed",
		"Fira Sans Extra Condensed;FiraSansExtraCondensed-Italic;Fira Sans;italic;400;extra-condensed",
		"Fira Sans Extra Condensed;FiraSansExtraCondensed-BoldItalic;Fira Sans;italic;700;extra-condensed",
		"Fira Sans Extra Condensed ExtraBold;FiraSansExtraCondensed-ExtraBold;Fira Sans;normal;800;extra-condensed",
		"Fira Sans Extra Condensed ExtraLight;FiraSansExtraCondensed-ExtraLight;Fira Sans;normal;200;extra-condensed",
		"Fira Sans Extra Condensed Medium;FiraSansExtraCondensed-MediumItalic;Fira Sans;italic;500;extra-condensed",
		"Fira Sans Extra Condensed ExtraBold;FiraSansExtraCondensed-ExtraBoldItalic;Fira Sans;italic;800;extra-condensed",
		"Fira Sans Extra Condensed Light;FiraSansExtraCondensed-LightItalic;Fira Sans;italic;300;extra-condensed",
		"Fira Sans Extra Condensed;FiraSansExtraCondensed-Regular;Fira Sans;normal;400;extra-condensed",
		"Fira Sans Condensed Black;FiraSansCondensed-BlackItalic;Fira Sans;italic;900;condensed",
		"Fira Sans Condensed Medium;FiraSansCondensed-Medium;Fira Sans;normal;500;condensed",
		"Fira Sans Condensed SemiBold;FiraSansCondensed-SemiBoldItalic;Fira Sans;italic;600;condensed",
		"Fira Sans Condensed;FiraSansCondensed-Regular;Fira Sans;normal;400;condensed",
		"Fira Sans Condensed;FiraSansCondensed-Bold;Fira Sans;normal;700;condensed",
		"Fira Sans Condensed Black;FiraSansCondensed-Black;Fira Sans;normal;900;condensed",
		"Fira Sans Condensed Thin;FiraSansCondensed-Thin;Fira Sans;normal;100;condensed",
		"Fira Sans Condensed Medium;FiraSansCondensed-MediumItalic;Fira Sans;italic;500;condensed",
		"Fira Sans Condensed ExtraBold;FiraSansCondensed-ExtraBoldItalic;Fira Sans;italic;800;condensed",
		"Fira Sans Condensed Light;FiraSansCondensed-LightItalic;Fira Sans;italic;300;condensed",
		"Fira Sans Condensed Thin;FiraSansCondensed-ThinItalic;Fira Sans;italic;100;condensed",
		"Fira Sans Condensed Light;FiraSansCondensed-Light;Fira Sans;normal;300;condensed",
		"Fira Sans Condensed;FiraSansCondensed-Italic;Fira Sans;italic;400;condensed",
		"Fira Sans Condensed ExtraLight;FiraSansCondensed-ExtraLight;Fira Sans;normal;200;condensed",
		"Fira Sans Condensed ExtraBold;FiraSansCondensed-ExtraBold;Fira Sans;normal;800;condensed",
		"Fira Sans Condensed;FiraSansCondensed-BoldItalic;Fira Sans;italic;700;condensed",
		"Fira Sans Condensed SemiBold;FiraSansCondensed-SemiBold;Fira Sans;normal;600;condensed",
		"Fira Sans Condensed ExtraLight;FiraSansCondensed-ExtraLightItalic;Fira Sans;italic;200;condensed",
		"PT Sans;PTSans-Regular;PT Sans;normal;400;normal",
		"PT Sans;PTSans-BoldItalic;PT Sans;italic;700;normal",
		"PT Sans;PTSans-Italic;PT Sans;italic;400;normal",
		"PT Sans;PTSans-Bold;PT Sans;normal;700;normal",
		"Nunito Black;Nunito-BlackItalic;Nunito;italic;900;normal",
		"Nunito;Nunito-Italic;Nunito;italic;400;normal",
		"Nunito SemiBold;Nunito-SemiBold;Nunito;normal;600;normal",
		"Nunito Light;Nunito-Light;Nunito;normal;300;normal",
		"Nunito ExtraBold;Nunito-ExtraBoldItalic;Nunito;italic;800;normal",
		"Nunito ExtraLight;Nunito-ExtraLightItalic;Nunito;italic;200;normal",
		"Nunito;Nunito-Regular;Nunito;normal;400;normal",
		"Nunito SemiBold;Nunito-SemiBoldItalic;Nunito;italic;600;normal",
		"Nunito;Nunito-Bold;Nunito;normal;700;normal",
		"Nunito;Nunito-BoldItalic;Nunito;italic;700;normal",
		"Nunito ExtraLight;Nunito-ExtraLight;Nunito;normal;200;normal",
		"Nunito ExtraBold;Nunito-ExtraBold;Nunito;normal;800;normal",
		"Nunito Black;Nunito-Black;Nunito;normal;900;normal",
		"Nunito Medium;Nunito-MediumItalic;Nunito;italic;500;normal",
		"Nunito Light;Nunito-LightItalic;Nunito;italic;300;normal",
		"Nunito Medium;Nunito-Medium;Nunito;normal;500;normal",
		"Roboto Condensed;RobotoCondensed-Bold;Roboto;normal;700;condensed",
		"Roboto Condensed Thin;RobotoCondensed-Thin;Roboto;normal;100;condensed",
		"Roboto Condensed ExtraBold;RobotoCondensed-ExtraBold;Roboto;normal;800;condensed",
		"Roboto Condensed;RobotoCondensed-Italic;Roboto;italic;400;condensed",
		"Roboto Condensed Light;RobotoCondensed-LightItalic;Roboto;italic;300;condensed",
		"Roboto Condensed Light;RobotoCondensed-Light;Roboto;normal;300;condensed",
		"Roboto Condensed Medium;RobotoCondensed-MediumItalic;Roboto;italic;500;condensed",
		"Roboto Condensed ExtraBold;RobotoCondensed-ExtraBoldItalic;Roboto;italic;800;condensed",
		"Roboto Condensed ExtraLight;RobotoCondensed-ExtraLightItalic;Roboto;italic;200;condensed",
		"Roboto Condensed;RobotoCondensed-Regular;Roboto;normal;400;condensed",
		"Roboto Condensed Thin;RobotoCondensed-ThinItalic;Roboto;italic;100;condensed",
		"Roboto Condensed Black;RobotoCondensed-Black;Roboto;normal;900;condensed",
		"Roboto Condensed SemiBold;RobotoCondensed-SemiBold;Roboto;normal;600;condensed",
		"Roboto Condensed SemiBold;RobotoCondensed-SemiBoldItalic;Roboto;italic;600;condensed",
		"Roboto Condensed Black;RobotoCondensed-BlackItalic;Roboto;italic;900;condensed",
		"Roboto Condensed Medium;RobotoCondensed-Medium;Roboto;normal;500;condensed",
		"Roboto Condensed;RobotoCondensed-BoldItalic;Roboto;italic;700;condensed",
		"Roboto Condensed ExtraLight;RobotoCondensed-ExtraLight;Roboto;normal;200;condensed",
		"Fira Sans;FiraSans-Regular;Fira Sans;normal;400;normal",
		"Fira Sans ExtraLight;FiraSans-ExtraLight;Fira Sans;normal;200;normal",
		"Fira Sans Medium;FiraSans-MediumItalic;Fira Sans;italic;500;normal",
		"Fira Sans;FiraSans-BoldItalic;Fira Sans;italic;700;normal",
		"Fira Sans Medium;FiraSans-Medium;Fira Sans;normal;500;normal",
		"Fira Sans Black;FiraSans-BlackItalic;Fira Sans;italic;900;normal",
		"Fira Sans Thin;FiraSans-ThinItalic;Fira Sans;italic;100;normal",
		"Fira Sans Light;FiraSans-Light;Fira Sans;normal;300;normal",
		"Fira Sans ExtraLight;FiraSans-ExtraLightItalic;Fira Sans;italic;200;normal",
		"Fira Sans;FiraSans-Bold;Fira Sans;normal;700;normal",
		"Fira Sans Black;FiraSans-Black;Fira Sans;normal;900;normal",
		"Fira Sans SemiBold;FiraSans-SemiBold;Fira Sans;normal;600;normal",
		"Fira Sans ExtraBold;FiraSans-ExtraBoldItalic;Fira Sans;italic;800;normal",
		"Fira Sans Thin;FiraSans-Thin;Fira Sans;normal;100;normal",
		"Fira Sans SemiBold;FiraSans-SemiBoldItalic;Fira Sans;italic;600;normal",
		"Fira Sans Light;FiraSans-LightItalic;Fira Sans;italic;300;normal",
		"Fira Sans ExtraBold;FiraSans-ExtraBold;Fira Sans;normal;800;normal",
		"Fira Sans;FiraSans-Italic;Fira Sans;italic;400;normal",
		"Roboto;Roboto-BoldItalic;Roboto;italic;700;normal",
		"Roboto Light;Roboto-Light;Roboto;normal;300;normal",
		"Roboto Light;Roboto-LightItalic;Roboto;italic;300;normal",
		"Roboto Medium;Roboto-Medium;Roboto;normal;500;normal",
		"Roboto Thin;Roboto-ThinItalic;Roboto;italic;100;normal",
		"Roboto;Roboto-Regular;Roboto;normal;400;normal",
		"Roboto;Roboto-Italic;Roboto;italic;400;normal",
		"Roboto;Roboto-Bold;Roboto;normal;700;normal",
		"Roboto Thin;Roboto-Thin;Roboto;normal;100;normal",
		"Roboto Medium;Roboto-MediumItalic;Roboto;italic;500;normal",
		"Roboto Black;Roboto-BlackItalic;Roboto;italic;900;normal",
		"Roboto Black;Roboto-Black;Roboto;normal;900;normal",
		"PT Sans Caption;PTSans-Caption;PT Sans Caption;normal;400;normal",
		"PT Sans Caption;PTSans-CaptionBold;PT Sans Caption;normal;700;normal",
		"PT Sans Narrow;PTSans-NarrowBold;PT Sans Narrow;normal;700;normal",
		"PT Sans Narrow;PTSans-Narrow;PT Sans Narrow;normal;400;normal",
		"Open Sans Condensed Medium;OpenSansCondensed-Medium;Open Sans;normal;500;condensed",
		"Open Sans Condensed;OpenSansCondensed-Regular;Open Sans;normal;400;condensed",
		"Open Sans Condensed Light;OpenSansCondensed-Light;Open Sans;normal;300;condensed",
		"Open Sans Condensed ExtraBold;OpenSansCondensed-ExtraBold;Open Sans;normal;800;condensed",
		"Open Sans Condensed Medium;OpenSansCondensed-MediumItalic;Open Sans;italic;500;condensed",
		"Open Sans Condensed SemiBold;OpenSansCondensed-SemiBoldItalic;Open Sans;italic;600;condensed",
		"Open Sans Condensed;OpenSansCondensed-Bold;Open Sans;normal;700;condensed",
		"Open Sans Condensed Light;OpenSansCondensed-LightItalic;Open Sans;italic;300;condensed",
		"Open Sans Condensed ExtraBold;OpenSansCondensed-ExtraBoldItalic;Open Sans;italic;800;condensed",
		"Open Sans Condensed SemiBold;OpenSansCondensed-SemiBold;Open Sans;normal;600;condensed",
		"Open Sans Condensed;OpenSansCondensed-Italic;Open Sans;italic;400;condensed",
		"Open Sans Condensed;OpenSansCondensed-BoldItalic;Open Sans;italic;700;condensed",
		"Merriweather Sans Light;MerriweatherSans-Light;Merriweather Sans;normal;300;normal",
		"Merriweather Sans;MerriweatherSans-BoldItalic;Merriweather Sans;italic;700;normal",
		"Merriweather Sans ExtraBold;MerriweatherSans-ExtraBoldItalic;Merriweather Sans;italic;800;normal",
		"Merriweather Sans ExtraBold;MerriweatherSans-ExtraBold;Merriweather Sans;normal;800;normal",
		"Merriweather Sans;MerriweatherSans-Italic;Merriweather Sans;italic;400;normal",
		"Merriweather Sans SemiBold;MerriweatherSans-SemiBoldItalic;Merriweather Sans;italic;600;normal",
		"Merriweather Sans;MerriweatherSans-Regular;Merriweather Sans;normal;400;normal",
		"Merriweather Sans Light;MerriweatherSans-LightItalic;Merriweather Sans;italic;300;normal",
		"Merriweather Sans SemiBold;MerriweatherSans-SemiBold;Merriweather Sans;normal;600;normal",
		"Merriweather Sans Medium;MerriweatherSans-MediumItalic;Merriweather Sans;italic;500;normal",
		"Merriweather Sans Medium;MerriweatherSans-Medium;Merriweather Sans;normal;500;normal",
		"Merriweather Sans;MerriweatherSans-Bold;Merriweather Sans;normal;700;normal"
	];

		for sample in samples {
			let parts = sample.split(';').collect::<Vec<&str>>();
			let (family, style, weight, width) =
				parse_font_name(parts[0].to_owned(), parts[1].to_owned());
			assert_eq!(family, parts[2]);
			assert_eq!(style, parts[3]);
			assert_eq!(weight.to_string(), parts[4]);
			assert_eq!(width, parts[5]);
		}
	}
}
