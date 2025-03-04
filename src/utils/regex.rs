use regex_lite::Regex;

pub fn replace_all(text: &str, regex: &str, replacement: &str) -> String {
	let re = Regex::new(regex).unwrap();
	re.replace_all(text, replacement).to_string()
}
