use icu_properties::{CodePointMapData, props::Script};

pub fn detect<S: AsRef<str>>(words: &[S]) -> &'static str {
	// icu 2.x: const-fn singleton over baked data; constructing per call is free.
	let script = CodePointMapData::<Script>::new();
	for word in words {
		for ch in word.as_ref().chars() {
			match script.get(ch) {
				Script::Hebrew => return "he",
				// Ancient Greek (ISO 639-3): these are biblical-language
				// dictionaries, and "grc" has no 639-1 equivalent.
				Script::Greek => return "grc",
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
		// Ancient Greek: ISO 639-3 "grc" (no 639-1 code exists), not modern "el".
		assert_eq!(detect(&["λογος"]), "grc");
	}

	#[test]
	fn unknown_for_latin() {
		assert_eq!(detect(&["word"]), "unknown");
	}
}
