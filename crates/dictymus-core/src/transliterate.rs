type Plane = fn(char) -> Option<&'static str>;

/// What the Logos Biblical keyboard layout enters for a typed key: the
/// Shift-plane entry for the key as typed, else the plain entry for its
/// lowercase form. None when the layout enters nothing script-specific.
pub fn transliterate(ch: char, language: &str) -> Option<&'static str> {
	let (shifted, plain): (Plane, Plane) = match language {
		"he" => (hebrew_shift, hebrew),
		"grc" => (greek_shift, greek),
		_ => return None,
	};
	shifted(ch).or_else(|| plain(ch.to_ascii_lowercase()))
}

fn hebrew(c: char) -> Option<&'static str> {
	Some(match c {
		'\'' => "א",
		'`' => "ע",
		'a' => "\u{5b7}", // patah
		'b' => "ב",
		'c' => "צ",
		'd' => "ד",
		'e' => "\u{5b6}", // segol
		'f' => "ט",
		'g' => "ג",
		'h' => "ה",
		'i' => "\u{5b4}", // hiriq
		'j' => "ח",
		'k' => "כ",
		'l' => "ל",
		'm' => "מ",
		'n' => "נ",
		'o' => "\u{5b8}", // qamats
		'p' => "פ",
		'q' => "ק",
		'r' => "ר",
		's' => "ס",
		't' => "ת",
		'u' => "\u{5bb}", // qubuts
		'v' | 'w' => "ו",
		'x' => "ש",
		'y' => "י",
		'z' => "ז",
		',' => "\u{5b0}", // sheva
		'.' => "\u{5bc}", // dagesh
		'-' => "־",
		';' => "׃",
		'\\' => "׀",
		'[' => "שׂ",
		']' => "שׁ",
		_ => return None,
	})
}

fn hebrew_shift(c: char) -> Option<&'static str> {
	Some(match c {
		'"' => "ע",
		'~' => "\u{5bf}", // rafe
		'A' => "\u{5b8}", // qamats
		'C' => "ץ",
		'D' => "\u{5b1}", // hataf segol
		'E' => "\u{5b5}", // tsere
		'H' => "ח",
		'I' => "\u{5b4}", // hiriq
		'K' => "ך",
		'L' => "\u{5b3}", // hataf qamats
		'M' => "ם",
		'N' => "ן",
		'O' => "\u{5b9}", // holam
		'P' => "ף",
		'S' => "ש",
		'T' => "ט",
		'U' => "וּ",
		'Z' => "\u{5b2}", // hataf patah
		':' => "׃",
		'<' => "\u{5b0}", // sheva
		'>' => "\u{5bc}", // dagesh
		'+' => "\u{5af}", // masora circle
		'}' => "\u{5bd}", // meteg
		'|' => "׀",
		_ => return None,
	})
}

fn greek(c: char) -> Option<&'static str> {
	Some(match c {
		'a' => "α",
		'b' => "β",
		'g' => "γ",
		'd' => "δ",
		'e' => "ε",
		'z' => "ζ",
		'h' => "η",
		'q' => "θ",
		'i' => "ι",
		'k' => "κ",
		'l' => "λ",
		'm' => "μ",
		'n' => "ν",
		'x' => "ξ",
		'o' => "ο",
		'p' => "π",
		'r' => "ρ",
		's' => "σ",
		't' => "τ",
		'u' => "υ",
		'f' => "φ",
		'c' => "χ",
		'y' => "ψ",
		'w' => "ω",
		'v' => "ς",
		'j' => "\u{345}",  // iota subscript
		'/' => "\u{301}",  // acute
		'\\' => "\u{300}", // grave
		'=' => "\u{342}",  // circumflex
		'[' => "\u{314}",  // rough breathing
		']' => "\u{313}",  // smooth breathing
		'`' => "\u{308}",  // diaeresis
		_ => return None,
	})
}

fn greek_shift(c: char) -> Option<&'static str> {
	Some(match c {
		'A' => "Α",
		'B' => "Β",
		'G' => "Γ",
		'D' => "Δ",
		'E' => "Ε",
		'Z' => "Ζ",
		'H' => "Η",
		'Q' => "Θ",
		'I' => "Ι",
		'K' => "Κ",
		'L' => "Λ",
		'M' => "Μ",
		'N' => "Ν",
		'X' => "Ξ",
		'O' => "Ο",
		'P' => "Π",
		'R' => "Ρ",
		'S' | 'V' => "Σ",
		'T' => "Τ",
		'U' => "Υ",
		'F' => "Φ",
		'C' => "Χ",
		'Y' => "Ψ",
		'W' => "Ω",
		'!' => "\u{304}", // macron
		'@' => "\u{306}", // breve
		'%' => "\u{307}", // dot above
		'^' => "\u{323}", // dot below
		'&' => "ϗ",
		_ => return None,
	})
}

#[cfg(test)]
mod tests {
	use super::transliterate;

	fn he(ch: char) -> Option<&'static str> {
		transliterate(ch, "he")
	}

	fn grc(ch: char) -> Option<&'static str> {
		transliterate(ch, "grc")
	}

	#[test]
	fn hebrew_consonants() {
		assert_eq!(he('b'), Some("ב"));
		assert_eq!(he('\''), Some("א"));
		assert_eq!(he('`'), Some("ע"));
		assert_eq!(he('"'), Some("ע"));
		assert_eq!(he('f'), Some("ט"));
		assert_eq!(he('x'), Some("ש"));
	}

	#[test]
	fn hebrew_plain_vowels_and_points() {
		assert_eq!(he('a'), Some("\u{5b7}")); // patah
		assert_eq!(he('e'), Some("\u{5b6}")); // segol
		assert_eq!(he('i'), Some("\u{5b4}")); // hiriq
		assert_eq!(he('o'), Some("\u{5b8}")); // qamats, not holam
		assert_eq!(he('u'), Some("\u{5bb}")); // qubuts
		assert_eq!(he(','), Some("\u{5b0}")); // sheva
		assert_eq!(he('.'), Some("\u{5bc}")); // dagesh
	}

	#[test]
	fn hebrew_shift_plane() {
		assert_eq!(he('A'), Some("\u{5b8}")); // qamats
		assert_eq!(he('E'), Some("\u{5b5}")); // tsere
		assert_eq!(he('O'), Some("\u{5b9}")); // holam
		assert_eq!(he('I'), Some("\u{5b4}")); // hiriq
		assert_eq!(he('D'), Some("\u{5b1}")); // hataf segol
		assert_eq!(he('L'), Some("\u{5b3}")); // hataf qamats
		assert_eq!(he('Z'), Some("\u{5b2}")); // hataf patah
		assert_eq!(he('M'), Some("ם"));
		assert_eq!(he('K'), Some("ך"));
		assert_eq!(he('N'), Some("ן"));
		assert_eq!(he('P'), Some("ף"));
		assert_eq!(he('C'), Some("ץ"));
		assert_eq!(he('H'), Some("ח"));
		assert_eq!(he('T'), Some("ט"));
		assert_eq!(he('S'), Some("ש"));
		assert_eq!(he('U'), Some("וּ")); // shureq
		assert_eq!(he('~'), Some("\u{5bf}")); // rafe
		assert_eq!(he('<'), Some("\u{5b0}"));
		assert_eq!(he('>'), Some("\u{5bc}"));
		assert_eq!(he('+'), Some("\u{5af}")); // masora circle
		assert_eq!(he('}'), Some("\u{5bd}")); // meteg
		assert_eq!(he(':'), Some("׃"));
		assert_eq!(he('|'), Some("׀"));
	}

	#[test]
	fn hebrew_shin_keys_carry_their_dot() {
		assert_eq!(he('['), Some("שׂ"));
		assert_eq!(he(']'), Some("שׁ"));
	}

	#[test]
	fn hebrew_punctuation() {
		assert_eq!(he('-'), Some("־"));
		assert_eq!(he(';'), Some("׃"));
		assert_eq!(he('\\'), Some("׀"));
	}

	#[test]
	fn hebrew_uppercase_without_shift_mapping_falls_back_to_plain() {
		assert_eq!(he('B'), Some("ב"));
		assert_eq!(he('Q'), Some("ק"));
		assert_eq!(he('W'), Some("ו"));
	}

	#[test]
	fn hebrew_unmapped_keys_pass_through() {
		assert_eq!(he('('), None);
		assert_eq!(he('1'), None);
		assert_eq!(he('_'), None);
		assert_eq!(he('='), None);
		assert_eq!(he('{'), None);
	}

	#[test]
	fn greek_plain_plane() {
		assert_eq!(grc('q'), Some("θ"));
		assert_eq!(grc('c'), Some("χ"));
		assert_eq!(grc('x'), Some("ξ"));
		assert_eq!(grc('v'), Some("ς"));
		assert_eq!(grc('j'), Some("\u{345}")); // iota subscript
		assert_eq!(grc('/'), Some("\u{301}")); // acute
		assert_eq!(grc('\\'), Some("\u{300}")); // grave
		assert_eq!(grc('='), Some("\u{342}")); // circumflex
		assert_eq!(grc('['), Some("\u{314}")); // rough breathing
		assert_eq!(grc(']'), Some("\u{313}")); // smooth breathing
		assert_eq!(grc('`'), Some("\u{308}")); // diaeresis
	}

	#[test]
	fn greek_shift_plane() {
		assert_eq!(grc('A'), Some("Α"));
		assert_eq!(grc('V'), Some("Σ"));
		assert_eq!(grc('W'), Some("Ω"));
		assert_eq!(grc('!'), Some("\u{304}")); // macron
		assert_eq!(grc('@'), Some("\u{306}")); // breve
		assert_eq!(grc('%'), Some("\u{307}")); // dot above
		assert_eq!(grc('^'), Some("\u{323}")); // dot below
		assert_eq!(grc('&'), Some("ϗ"));
	}

	#[test]
	fn greek_unmapped_keys_pass_through() {
		assert_eq!(grc('\''), None);
		assert_eq!(grc('-'), None);
		assert_eq!(grc('1'), None);
		assert_eq!(grc(';'), None);
		assert_eq!(grc('#'), None);
	}

	#[test]
	fn unknown_language_passes_through() {
		assert_eq!(transliterate('b', "unknown"), None);
	}
}
