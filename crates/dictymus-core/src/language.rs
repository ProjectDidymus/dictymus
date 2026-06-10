use icu_properties::{props::Script, CodePointMapData};

pub fn detect<S: AsRef<str>>(words: &[S]) -> &'static str {
	// icu 2.x: const-fn singleton over baked data; constructing per call is free.
	let script = CodePointMapData::<Script>::new();
	for word in words {
		for ch in word.as_ref().chars() {
			match script.get(ch) {
				Script::Hebrew => return "he",
				Script::Greek => return "el",
				_ => {}
			}
		}
	}
	"unknown"
}

#[cfg(test)]
mod tests {
	use super::detect;

	#[test]
	fn detects_hebrew() {
		assert_eq!(detect(&["דבר", "בית"]), "he");
	}

	#[test]
	fn detects_greek() {
		assert_eq!(detect(&["λογος"]), "el");
	}

	#[test]
	fn unknown_for_latin() {
		assert_eq!(detect(&["word"]), "unknown");
	}
}
