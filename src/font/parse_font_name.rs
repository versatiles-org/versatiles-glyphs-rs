pub fn parse_font_name(family: String, ps_name: String) -> (String, String, u16, String) {
	// 1) Default values
	let mut style = "normal".to_string();
	let mut weight = 400u16;
	let mut width = "normal".to_string();

	let suffix = if let Some(pos) = ps_name.rfind('-') {
		&ps_name[pos + 1..] // substring after the dash
	} else {
		&ps_name // if no dash, treat entire ps_name as suffix
	};
	let lower_suffix = suffix.to_lowercase();

	// Style detection
	if lower_suffix.contains("italic") {
		style = "italic".to_string();
	}

	// Weight detection from the PS-suffix
	let ps_weight = find_weight(&lower_suffix);
	if ps_weight != 400 {
		weight = ps_weight;
	}

	// ------------------------------------------------------
	// 3) Parse the "family" string to remove known width/weight tokens,
	//    leaving a cleaned-up family name. Also parse out width or weight
	//    from the family if they appear there.
	// ------------------------------------------------------
	let tokens = family.split_whitespace().collect::<Vec<&str>>();
	let mut final_family_tokens = Vec::new();

	let mut i = 0;
	while i < tokens.len() {
		let t = tokens[i].to_lowercase();

		// -- (a) Check for "Extra Condensed" as a two-word width -------------
		if i + 1 < tokens.len() && t == "extra" && tokens[i + 1].to_lowercase() == "condensed" {
			width = "extra condensed".to_string();
			i += 2;
			continue;
		}

		// -- (b) Check single-token widths -----------------------------------
		if t == "semicondensed" {
			width = "semicondensed".to_string();
			i += 1;
			continue;
		}
		if t == "condensed" {
			width = "condensed".to_string();
			i += 1;
			continue;
		}
		if t == "caption" {
			width = "caption".to_string();
			i += 1;
			continue;
		}
		if t == "narrow" {
			width = "narrow".to_string();
			i += 1;
			continue;
		}

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

		// -- (c) Check if this token corresponds to a known weight ----------
		let maybe_w = find_weight(&t);
		if maybe_w != 400 {
			// Found a weight token; only use it if we haven't already
			// overridden from the PS name suffix
			if ps_weight == 400 {
				weight = maybe_w;
			}
			i += 1;
			continue;
		}

		// If none of the above, it's a real part of the family
		final_family_tokens.push(tokens[i]);
		i += 1;
	}

	let final_family = final_family_tokens.join(" ");
	(final_family, style, weight, width)
}

// ------------------------------------------------------
// Helper for parsing weight from a lowercase token
// ------------------------------------------------------
fn find_weight(s: &str) -> u16 {
	// NOTE: order these checks from most-specific to least-specific
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
		// cover "bold", "extrabold", "ultrabold"
		if s.contains("extra") || s.contains("ultra") {
			800
		} else {
			700
		}
	} else if s.contains("black") || s.contains("heavy") {
		900
	} else {
		// Default to 400 if we can't detect any known weight
		400
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_font_name() {
		fn test(input: &str) {
			let input = input.split(';').collect::<Vec<&str>>();
			assert_eq!(input.len(), 6);
			let (family, style, weight, width) =
				parse_font_name(input[0].to_owned(), input[1].to_owned());
			assert_eq!(family, input[2]);
			assert_eq!(style, input[3]);
			assert_eq!(weight.to_string(), input[4]);
			assert_eq!(width, input[5]);
		}
		test("Open Sans SemiCondensed ExtraBold;OpenSansSemiCondensed-ExtraBold;Open Sans;normal;800;semicondensed");
		test("Open Sans SemiCondensed Light;OpenSansSemiCondensed-LightItalic;Open Sans;italic;300;semicondensed");
		test("Open Sans SemiCondensed SemiBold;OpenSansSemiCondensed-SemiBold;Open Sans;normal;600;semicondensed");
		test(
			"Open Sans SemiCondensed;OpenSansSemiCondensed-Italic;Open Sans;italic;400;semicondensed",
		);
		test("Open Sans SemiCondensed Medium;OpenSansSemiCondensed-MediumItalic;Open Sans;italic;500;semicondensed");
		test("Open Sans SemiCondensed ExtraBold;OpenSansSemiCondensed-ExtraBoldItalic;Open Sans;italic;800;semicondensed");
		test("Open Sans SemiCondensed;OpenSansSemiCondensed-Bold;Open Sans;normal;700;semicondensed");
		test("Open Sans SemiCondensed;OpenSansSemiCondensed-BoldItalic;Open Sans;italic;700;semicondensed");
		test("Open Sans SemiCondensed Medium;OpenSansSemiCondensed-Medium;Open Sans;normal;500;semicondensed");
		test("Open Sans SemiCondensed SemiBold;OpenSansSemiCondensed-SemiBoldItalic;Open Sans;italic;600;semicondensed");
		test(
			"Open Sans SemiCondensed;OpenSansSemiCondensed-Regular;Open Sans;normal;400;semicondensed",
		);
		test("Open Sans SemiCondensed Light;OpenSansSemiCondensed-Light;Open Sans;normal;300;semicondensed");
		test("Open Sans;OpenSans-BoldItalic;Open Sans;italic;700;normal");
		test("Open Sans Medium;OpenSans-Medium;Open Sans;normal;500;normal");
		test("Open Sans Medium;OpenSans-MediumItalic;Open Sans;italic;500;normal");
		test("Open Sans ExtraBold;OpenSans-ExtraBoldItalic;Open Sans;italic;800;normal");
		test("Open Sans SemiBold;OpenSans-SemiBold;Open Sans;normal;600;normal");
		test("Open Sans;OpenSans-Bold;Open Sans;normal;700;normal");
		test("Open Sans ExtraBold;OpenSans-ExtraBold;Open Sans;normal;800;normal");
		test("Open Sans Light;OpenSans-LightItalic;Open Sans;italic;300;normal");
		test("Open Sans;OpenSans-Italic;Open Sans;italic;400;normal");
		test("Open Sans Light;OpenSans-Light;Open Sans;normal;300;normal");
		test("Open Sans SemiBold;OpenSans-SemiBoldItalic;Open Sans;italic;600;normal");
		test("Open Sans;OpenSans-Regular;Open Sans;normal;400;normal");
		test("Libre Baskerville;LibreBaskerville-Regular;Libre Baskerville;normal;400;normal");
		test("Libre Baskerville;LibreBaskerville-Bold;Libre Baskerville;normal;700;normal");
		test("Libre Baskerville;LibreBaskerville-Italic;Libre Baskerville;italic;400;normal");
		test("Noto Sans;NotoSans-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Arabic;NotoSansArabic-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Armenian;NotoSansArmenian-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Balinese;NotoSansBalinese-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Bengali;NotoSansBengali-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Devanagari;NotoSansDevanagari-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Ethiopic;NotoSansEthiopic-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Georgian;NotoSansGeorgian-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Gujarati;NotoSansGujarati-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Gurmukhi;NotoSansGurmukhi-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Hebrew;NotoSansHebrew-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans JP;NotoSansJP-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Javanese;NotoSansJavanese-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans KR;NotoSansKR-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Kannada;NotoSansKannada-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Khmer;NotoSansKhmer-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Lao;NotoSansLao-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Myanmar;NotoSansMyanmar-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Oriya;NotoSansOriya-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans SC;NotoSansSC-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Sinhala;NotoSansSinhala-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Tamil;NotoSansTamil-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans Thai;NotoSansThai-Regular;Noto Sans;normal;400;normal");
		test("Noto Sans;NotoSans-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Arabic;NotoSansArabic-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Armenian;NotoSansArmenian-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Balinese;NotoSansBalinese-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Bengali;NotoSansBengali-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Devanagari;NotoSansDevanagari-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Ethiopic;NotoSansEthiopic-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Georgian;NotoSansGeorgian-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Gujarati;NotoSansGujarati-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Gurmukhi;NotoSansGurmukhi-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Hebrew;NotoSansHebrew-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans JP;NotoSansJP-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Javanese;NotoSansJavanese-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans KR;NotoSansKR-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Kannada;NotoSansKannada-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Khmer;NotoSansKhmer-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Lao;NotoSansLao-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Myanmar;NotoSansMyanmar-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Oriya;NotoSansOriya-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans SC;NotoSansSC-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Sinhala;NotoSansSinhala-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Tamil;NotoSansTamil-Bold;Noto Sans;normal;700;normal");
		test("Noto Sans Thai;NotoSansThai-Bold;Noto Sans;normal;700;normal");
		test("Lato;Lato-Regular;Lato;normal;400;normal");
		test("Lato Light;Lato-LightItalic;Lato;italic;300;normal");
		test("Lato;Lato-Italic;Lato;italic;400;normal");
		test("Lato;Lato-Bold;Lato;normal;700;normal");
		test("Lato Hairline;Lato-Hairline;Lato;normal;100;normal");
		test("Lato Light;Lato-Light;Lato;normal;300;normal");
		test("Lato Black;Lato-BlackItalic;Lato;italic;900;normal");
		test("Lato Black;Lato-Black;Lato;normal;900;normal");
		test("Lato Hairline;Lato-HairlineItalic;Lato;italic;100;normal");
		test("Lato;Lato-BoldItalic;Lato;italic;700;normal");
		test("Source Sans 3 Black;SourceSans3-BlackItalic;Source Sans 3;italic;900;normal");
		test("Source Sans 3 Black;SourceSans3-Black;Source Sans 3;normal;900;normal");
		test("Source Sans 3 Medium;SourceSans3-Medium;Source Sans 3;normal;500;normal");
		test("Source Sans 3;SourceSans3-Regular;Source Sans 3;normal;400;normal");
		test("Source Sans 3 ExtraLight;SourceSans3-ExtraLightItalic;Source Sans 3;italic;200;normal");
		test("Source Sans 3;SourceSans3-BoldItalic;Source Sans 3;italic;700;normal");
		test("Source Sans 3 ExtraLight;SourceSans3-ExtraLight;Source Sans 3;normal;200;normal");
		test("Source Sans 3 SemiBold;SourceSans3-SemiBold;Source Sans 3;normal;600;normal");
		test("Source Sans 3 ExtraBold;SourceSans3-ExtraBoldItalic;Source Sans 3;italic;800;normal");
		test("Source Sans 3 Light;SourceSans3-LightItalic;Source Sans 3;italic;300;normal");
		test("Source Sans 3 ExtraBold;SourceSans3-ExtraBold;Source Sans 3;normal;800;normal");
		test("Source Sans 3;SourceSans3-Bold;Source Sans 3;normal;700;normal");
		test("Source Sans 3;SourceSans3-Italic;Source Sans 3;italic;400;normal");
		test("Source Sans 3 SemiBold;SourceSans3-SemiBoldItalic;Source Sans 3;italic;600;normal");
		test("Source Sans 3 Medium;SourceSans3-MediumItalic;Source Sans 3;italic;500;normal");
		test("Source Sans 3 Light;SourceSans3-Light;Source Sans 3;normal;300;normal");
		test("Fira Sans Extra Condensed Medium;FiraSansExtraCondensed-Medium;Fira Sans;normal;500;extra condensed");
		test("Fira Sans Extra Condensed SemiBold;FiraSansExtraCondensed-SemiBold;Fira Sans;normal;600;extra condensed");
		test("Fira Sans Extra Condensed ExtraLight;FiraSansExtraCondensed-ExtraLightItalic;Fira Sans;italic;200;extra condensed");
		test("Fira Sans Extra Condensed Light;FiraSansExtraCondensed-Light;Fira Sans;normal;300;extra condensed");
		test("Fira Sans Extra Condensed Black;FiraSansExtraCondensed-BlackItalic;Fira Sans;italic;900;extra condensed");
		test("Fira Sans Extra Condensed Thin;FiraSansExtraCondensed-Thin;Fira Sans;normal;100;extra condensed");
		test("Fira Sans Extra Condensed SemiBold;FiraSansExtraCondensed-SemiBoldItalic;Fira Sans;italic;600;extra condensed");
		test("Fira Sans Extra Condensed Black;FiraSansExtraCondensed-Black;Fira Sans;normal;900;extra condensed");
		test("Fira Sans Extra Condensed Thin;FiraSansExtraCondensed-ThinItalic;Fira Sans;italic;100;extra condensed");
		test("Fira Sans Extra Condensed;FiraSansExtraCondensed-Bold;Fira Sans;normal;700;extra condensed");
		test("Fira Sans Extra Condensed;FiraSansExtraCondensed-Italic;Fira Sans;italic;400;extra condensed");
		test("Fira Sans Extra Condensed;FiraSansExtraCondensed-BoldItalic;Fira Sans;italic;700;extra condensed");
		test("Fira Sans Extra Condensed ExtraBold;FiraSansExtraCondensed-ExtraBold;Fira Sans;normal;800;extra condensed");
		test("Fira Sans Extra Condensed ExtraLight;FiraSansExtraCondensed-ExtraLight;Fira Sans;normal;200;extra condensed");
		test("Fira Sans Extra Condensed Medium;FiraSansExtraCondensed-MediumItalic;Fira Sans;italic;500;extra condensed");
		test("Fira Sans Extra Condensed ExtraBold;FiraSansExtraCondensed-ExtraBoldItalic;Fira Sans;italic;800;extra condensed");
		test("Fira Sans Extra Condensed Light;FiraSansExtraCondensed-LightItalic;Fira Sans;italic;300;extra condensed");
		test("Fira Sans Extra Condensed;FiraSansExtraCondensed-Regular;Fira Sans;normal;400;extra condensed");
		test(
			"Fira Sans Condensed Black;FiraSansCondensed-BlackItalic;Fira Sans;italic;900;condensed",
		);
		test("Fira Sans Condensed Medium;FiraSansCondensed-Medium;Fira Sans;normal;500;condensed");
		test("Fira Sans Condensed SemiBold;FiraSansCondensed-SemiBoldItalic;Fira Sans;italic;600;condensed");
		test("Fira Sans Condensed;FiraSansCondensed-Regular;Fira Sans;normal;400;condensed");
		test("Fira Sans Condensed;FiraSansCondensed-Bold;Fira Sans;normal;700;condensed");
		test("Fira Sans Condensed Black;FiraSansCondensed-Black;Fira Sans;normal;900;condensed");
		test("Fira Sans Condensed Thin;FiraSansCondensed-Thin;Fira Sans;normal;100;condensed");
		test(
			"Fira Sans Condensed Medium;FiraSansCondensed-MediumItalic;Fira Sans;italic;500;condensed",
		);
		test("Fira Sans Condensed ExtraBold;FiraSansCondensed-ExtraBoldItalic;Fira Sans;italic;800;condensed");
		test(
			"Fira Sans Condensed Light;FiraSansCondensed-LightItalic;Fira Sans;italic;300;condensed",
		);
		test("Fira Sans Condensed Thin;FiraSansCondensed-ThinItalic;Fira Sans;italic;100;condensed");
		test("Fira Sans Condensed Light;FiraSansCondensed-Light;Fira Sans;normal;300;condensed");
		test("Fira Sans Condensed;FiraSansCondensed-Italic;Fira Sans;italic;400;condensed");
		test("Fira Sans Condensed ExtraLight;FiraSansCondensed-ExtraLight;Fira Sans;normal;200;condensed");
		test(
			"Fira Sans Condensed ExtraBold;FiraSansCondensed-ExtraBold;Fira Sans;normal;800;condensed",
		);
		test("Fira Sans Condensed;FiraSansCondensed-BoldItalic;Fira Sans;italic;700;condensed");
		test(
			"Fira Sans Condensed SemiBold;FiraSansCondensed-SemiBold;Fira Sans;normal;600;condensed",
		);
		test("Fira Sans Condensed ExtraLight;FiraSansCondensed-ExtraLightItalic;Fira Sans;italic;200;condensed");
		test("PT Sans;PTSans-Regular;PT Sans;normal;400;normal");
		test("PT Sans;PTSans-BoldItalic;PT Sans;italic;700;normal");
		test("PT Sans;PTSans-Italic;PT Sans;italic;400;normal");
		test("PT Sans;PTSans-Bold;PT Sans;normal;700;normal");
		test("Nunito Black;Nunito-BlackItalic;Nunito;italic;900;normal");
		test("Nunito;Nunito-Italic;Nunito;italic;400;normal");
		test("Nunito SemiBold;Nunito-SemiBold;Nunito;normal;600;normal");
		test("Nunito Light;Nunito-Light;Nunito;normal;300;normal");
		test("Nunito ExtraBold;Nunito-ExtraBoldItalic;Nunito;italic;800;normal");
		test("Nunito ExtraLight;Nunito-ExtraLightItalic;Nunito;italic;200;normal");
		test("Nunito;Nunito-Regular;Nunito;normal;400;normal");
		test("Nunito SemiBold;Nunito-SemiBoldItalic;Nunito;italic;600;normal");
		test("Nunito;Nunito-Bold;Nunito;normal;700;normal");
		test("Nunito;Nunito-BoldItalic;Nunito;italic;700;normal");
		test("Nunito ExtraLight;Nunito-ExtraLight;Nunito;normal;200;normal");
		test("Nunito ExtraBold;Nunito-ExtraBold;Nunito;normal;800;normal");
		test("Nunito Black;Nunito-Black;Nunito;normal;900;normal");
		test("Nunito Medium;Nunito-MediumItalic;Nunito;italic;500;normal");
		test("Nunito Light;Nunito-LightItalic;Nunito;italic;300;normal");
		test("Nunito Medium;Nunito-Medium;Nunito;normal;500;normal");
		test("Roboto Condensed;RobotoCondensed-Bold;Roboto;normal;700;condensed");
		test("Roboto Condensed Thin;RobotoCondensed-Thin;Roboto;normal;100;condensed");
		test("Roboto Condensed ExtraBold;RobotoCondensed-ExtraBold;Roboto;normal;800;condensed");
		test("Roboto Condensed;RobotoCondensed-Italic;Roboto;italic;400;condensed");
		test("Roboto Condensed Light;RobotoCondensed-LightItalic;Roboto;italic;300;condensed");
		test("Roboto Condensed Light;RobotoCondensed-Light;Roboto;normal;300;condensed");
		test("Roboto Condensed Medium;RobotoCondensed-MediumItalic;Roboto;italic;500;condensed");
		test(
			"Roboto Condensed ExtraBold;RobotoCondensed-ExtraBoldItalic;Roboto;italic;800;condensed",
		);
		test(
			"Roboto Condensed ExtraLight;RobotoCondensed-ExtraLightItalic;Roboto;italic;200;condensed",
		);
		test("Roboto Condensed;RobotoCondensed-Regular;Roboto;normal;400;condensed");
		test("Roboto Condensed Thin;RobotoCondensed-ThinItalic;Roboto;italic;100;condensed");
		test("Roboto Condensed Black;RobotoCondensed-Black;Roboto;normal;900;condensed");
		test("Roboto Condensed SemiBold;RobotoCondensed-SemiBold;Roboto;normal;600;condensed");
		test("Roboto Condensed SemiBold;RobotoCondensed-SemiBoldItalic;Roboto;italic;600;condensed");
		test("Roboto Condensed Black;RobotoCondensed-BlackItalic;Roboto;italic;900;condensed");
		test("Roboto Condensed Medium;RobotoCondensed-Medium;Roboto;normal;500;condensed");
		test("Roboto Condensed;RobotoCondensed-BoldItalic;Roboto;italic;700;condensed");
		test("Roboto Condensed ExtraLight;RobotoCondensed-ExtraLight;Roboto;normal;200;condensed");
		test("Fira Sans;FiraSans-Regular;Fira Sans;normal;400;normal");
		test("Fira Sans ExtraLight;FiraSans-ExtraLight;Fira Sans;normal;200;normal");
		test("Fira Sans Medium;FiraSans-MediumItalic;Fira Sans;italic;500;normal");
		test("Fira Sans;FiraSans-BoldItalic;Fira Sans;italic;700;normal");
		test("Fira Sans Medium;FiraSans-Medium;Fira Sans;normal;500;normal");
		test("Fira Sans Black;FiraSans-BlackItalic;Fira Sans;italic;900;normal");
		test("Fira Sans Thin;FiraSans-ThinItalic;Fira Sans;italic;100;normal");
		test("Fira Sans Light;FiraSans-Light;Fira Sans;normal;300;normal");
		test("Fira Sans ExtraLight;FiraSans-ExtraLightItalic;Fira Sans;italic;200;normal");
		test("Fira Sans;FiraSans-Bold;Fira Sans;normal;700;normal");
		test("Fira Sans Black;FiraSans-Black;Fira Sans;normal;900;normal");
		test("Fira Sans SemiBold;FiraSans-SemiBold;Fira Sans;normal;600;normal");
		test("Fira Sans ExtraBold;FiraSans-ExtraBoldItalic;Fira Sans;italic;800;normal");
		test("Fira Sans Thin;FiraSans-Thin;Fira Sans;normal;100;normal");
		test("Fira Sans SemiBold;FiraSans-SemiBoldItalic;Fira Sans;italic;600;normal");
		test("Fira Sans Light;FiraSans-LightItalic;Fira Sans;italic;300;normal");
		test("Fira Sans ExtraBold;FiraSans-ExtraBold;Fira Sans;normal;800;normal");
		test("Fira Sans;FiraSans-Italic;Fira Sans;italic;400;normal");
		test("Roboto;Roboto-BoldItalic;Roboto;italic;700;normal");
		test("Roboto Light;Roboto-Light;Roboto;normal;300;normal");
		test("Roboto Light;Roboto-LightItalic;Roboto;italic;300;normal");
		test("Roboto Medium;Roboto-Medium;Roboto;normal;500;normal");
		test("Roboto Thin;Roboto-ThinItalic;Roboto;italic;100;normal");
		test("Roboto;Roboto-Regular;Roboto;normal;400;normal");
		test("Roboto;Roboto-Italic;Roboto;italic;400;normal");
		test("Roboto;Roboto-Bold;Roboto;normal;700;normal");
		test("Roboto Thin;Roboto-Thin;Roboto;normal;100;normal");
		test("Roboto Medium;Roboto-MediumItalic;Roboto;italic;500;normal");
		test("Roboto Black;Roboto-BlackItalic;Roboto;italic;900;normal");
		test("Roboto Black;Roboto-Black;Roboto;normal;900;normal");
		test("PT Sans Caption;PTSans-Caption;PT Sans;normal;400;caption");
		test("PT Sans Caption;PTSans-CaptionBold;PT Sans;normal;700;caption");
		test("PT Sans Narrow;PTSans-NarrowBold;PT Sans;normal;700;narrow");
		test("PT Sans Narrow;PTSans-Narrow;PT Sans;normal;400;narrow");
		test("Open Sans Condensed Medium;OpenSansCondensed-Medium;Open Sans;normal;500;condensed");
		test("Open Sans Condensed;OpenSansCondensed-Regular;Open Sans;normal;400;condensed");
		test("Open Sans Condensed Light;OpenSansCondensed-Light;Open Sans;normal;300;condensed");
		test(
			"Open Sans Condensed ExtraBold;OpenSansCondensed-ExtraBold;Open Sans;normal;800;condensed",
		);
		test(
			"Open Sans Condensed Medium;OpenSansCondensed-MediumItalic;Open Sans;italic;500;condensed",
		);
		test("Open Sans Condensed SemiBold;OpenSansCondensed-SemiBoldItalic;Open Sans;italic;600;condensed");
		test("Open Sans Condensed;OpenSansCondensed-Bold;Open Sans;normal;700;condensed");
		test(
			"Open Sans Condensed Light;OpenSansCondensed-LightItalic;Open Sans;italic;300;condensed",
		);
		test("Open Sans Condensed ExtraBold;OpenSansCondensed-ExtraBoldItalic;Open Sans;italic;800;condensed");
		test(
			"Open Sans Condensed SemiBold;OpenSansCondensed-SemiBold;Open Sans;normal;600;condensed",
		);
		test("Open Sans Condensed;OpenSansCondensed-Italic;Open Sans;italic;400;condensed");
		test("Open Sans Condensed;OpenSansCondensed-BoldItalic;Open Sans;italic;700;condensed");
		test("Merriweather Sans Light;MerriweatherSans-Light;Merriweather Sans;normal;300;normal");
		test("Merriweather Sans;MerriweatherSans-BoldItalic;Merriweather Sans;italic;700;normal");
		test("Merriweather Sans ExtraBold;MerriweatherSans-ExtraBoldItalic;Merriweather Sans;italic;800;normal");
		test("Merriweather Sans ExtraBold;MerriweatherSans-ExtraBold;Merriweather Sans;normal;800;normal");
		test("Merriweather Sans;MerriweatherSans-Italic;Merriweather Sans;italic;400;normal");
		test("Merriweather Sans SemiBold;MerriweatherSans-SemiBoldItalic;Merriweather Sans;italic;600;normal");
		test("Merriweather Sans;MerriweatherSans-Regular;Merriweather Sans;normal;400;normal");
		test(
			"Merriweather Sans Light;MerriweatherSans-LightItalic;Merriweather Sans;italic;300;normal",
		);
		test(
			"Merriweather Sans SemiBold;MerriweatherSans-SemiBold;Merriweather Sans;normal;600;normal",
		);
		test("Merriweather Sans Medium;MerriweatherSans-MediumItalic;Merriweather Sans;italic;500;normal");
		test("Merriweather Sans Medium;MerriweatherSans-Medium;Merriweather Sans;normal;500;normal");
		test("Merriweather Sans;MerriweatherSans-Bold;Merriweather Sans;normal;700;normal");
	}
}
