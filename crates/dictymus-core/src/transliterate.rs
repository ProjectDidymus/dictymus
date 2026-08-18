/// Map a Latin keystroke to a Hebrew/Greek glyph per the Logos Biblical
/// keyboard layout. Returns the input unchanged if no mapping applies.
pub fn transliterate_char(ch: char, language: &str) -> char {
	let lower = ch.to_ascii_lowercase();
	let mapped = match language {
		"he" => hebrew(lower),
		"grc" => greek(lower),
		_ => None,
	};
	mapped.unwrap_or(ch)
}

fn hebrew(c: char) -> Option<char> {
	Some(match c {
		'\'' => 'א',
		'`' => 'ע',
		'b' => 'ב',
		'g' => 'ג',
		'd' => 'ד',
		'h' => 'ה',
		'w' | 'v' => 'ו',
		'z' => 'ז',
		'j' => 'ח',
		'f' => 'ט',
		'y' => 'י',
		'k' => 'כ',
		'l' => 'ל',
		'm' => 'מ',
		'n' => 'נ',
		's' => 'ס',
		'p' => 'פ',
		'c' => 'צ',
		'q' => 'ק',
		'r' => 'ר',
		'x' => 'ש',
		't' => 'ת',
		_ => return None,
	})
}

fn greek(c: char) -> Option<char> {
	Some(match c {
		'a' => 'α',
		'b' => 'β',
		'g' => 'γ',
		'd' => 'δ',
		'e' => 'ε',
		'z' => 'ζ',
		'h' => 'η',
		'q' => 'θ',
		'i' => 'ι',
		'k' => 'κ',
		'l' => 'λ',
		'm' => 'μ',
		'n' => 'ν',
		'x' => 'ξ',
		'o' => 'ο',
		'p' => 'π',
		'r' => 'ρ',
		's' => 'σ',
		't' => 'τ',
		'u' => 'υ',
		'f' => 'φ',
		'c' => 'χ',
		'y' => 'ψ',
		'w' => 'ω',
		_ => return None,
	})
}

#[cfg(test)]
mod tests {
	use super::transliterate_char;

	#[test]
	fn hebrew_maps_b_to_bet() {
		assert_eq!(transliterate_char('b', "he"), 'ב');
	}

	#[test]
	fn greek_maps_q_to_theta() {
		assert_eq!(transliterate_char('q', "grc"), 'θ');
	}

	#[test]
	fn unknown_language_passthrough() {
		assert_eq!(transliterate_char('b', "unknown"), 'b');
	}

	#[test]
	fn unmapped_char_passthrough() {
		assert_eq!(transliterate_char('1', "he"), '1');
	}

	#[test]
	fn uppercase_is_lowercased_before_lookup() {
		assert_eq!(transliterate_char('B', "he"), 'ב');
	}
}
