//! Blanks the Rust constructs that `xgettext --language=C` mis-tokenizes:
//! lifetimes (`'a`, `'static`, `'_`), whose lone `'` it reads as the start of a
//! character constant; raw strings (`r"..."`, `r#"..."#`), whose `#` delimiters
//! it does not recognise; and literals spanning several source lines, which C
//! allows only when every line but the last ends in a backslash. Left alone, its
//! scan runs past the real end of the construct and can swallow later strings
//! and comments — including ones in a following file, since every file goes to a
//! single `xgettext` call.
//!
//! Single-line double-quoted literals and `//` and `/* */` comments pass through
//! byte for byte, so `t(` calls and their `TRANSLATORS` comments survive intact.
//! Every replacement is the same length as what it replaces and newlines are
//! kept, so line numbers never shift and a comment stays adjacent to the call
//! below it.

/// Sanitized source, plus any line where blanking removed a translatable
/// string.
pub struct Sanitized {
	pub text: String,
	/// 1-based lines holding a multi-line literal that is the argument of a
	/// `t(`/`nt(` call, and so was blanked out of the extraction.
	pub blanked_call_literals: Vec<usize>,
}

/// Replace lifetimes, raw strings and multi-line literals in `src` with spaces.
pub fn sanitize_for_xgettext(src: &str) -> Sanitized {
	let chars: Vec<char> = src.chars().collect();
	let n = chars.len();
	let mut out = String::with_capacity(src.len());
	let mut blanked_call_literals = Vec::new();
	let mut i = 0;
	while i < n {
		let c = chars[i];
		if c == '/' && i + 1 < n && chars[i + 1] == '/' {
			while i < n && chars[i] != '\n' {
				out.push(chars[i]);
				i += 1;
			}
		} else if c == '/' && i + 1 < n && chars[i + 1] == '*' {
			out.push('/');
			out.push('*');
			i += 2;
			let mut depth = 1usize;
			while i < n && depth > 0 {
				if i + 1 < n && chars[i] == '/' && chars[i + 1] == '*' {
					out.push('/');
					out.push('*');
					i += 2;
					depth += 1;
				} else if i + 1 < n && chars[i] == '*' && chars[i + 1] == '/' {
					out.push('*');
					out.push('/');
					i += 2;
					depth -= 1;
				} else {
					out.push(chars[i]);
					i += 1;
				}
			}
		} else if c == '"' {
			match string_literal_end(&chars, i) {
				Some(end) if chars[i..=end].contains(&'\n') => {
					if preceded_by_translation_call(&chars, i) {
						blanked_call_literals.push(line_of(&chars, i));
					}
					blank_span(&chars[i..=end], &mut out);
					i = end + 1;
				}
				Some(end) => {
					out.extend(&chars[i..=end]);
					i = end + 1;
				}
				None => {
					out.extend(&chars[i..]);
					i = n;
				}
			}
		} else if c == 'r' {
			if let Some(hashes) = raw_string_hash_count(&chars, i) {
				for _ in 0..(hashes + 2) {
					out.push(' ');
				}
				i += hashes + 2;
				blank_raw_string_body(&chars, &mut i, hashes, &mut out);
			} else {
				out.push(c);
				i += 1;
			}
		} else if c == '\'' {
			if let Some(end) = char_literal_end(&chars, i) {
				out.extend(&chars[i..=end]);
				i = end + 1;
			} else {
				out.push(' ');
				i += 1;
			}
		} else {
			out.push(c);
			i += 1;
		}
	}
	Sanitized { text: out, blanked_call_literals }
}

/// The index of the `"` closing the literal that opens at `chars[i]`, or `None`
/// when the file ends first.
fn string_literal_end(chars: &[char], i: usize) -> Option<usize> {
	let n = chars.len();
	let mut j = i + 1;
	while j < n {
		match chars[j] {
			'\\' => j += 2,
			'"' => return Some(j),
			_ => j += 1,
		}
	}
	None
}

/// Write `span` as spaces, keeping its newlines.
fn blank_span(span: &[char], out: &mut String) {
	for c in span {
		out.push(if *c == '\n' { '\n' } else { ' ' });
	}
}

/// The 1-based line of `chars[i]`.
fn line_of(chars: &[char], i: usize) -> usize {
	chars[..i].iter().filter(|c| **c == '\n').count() + 1
}

/// Whether the literal opening at `chars[i]` is the first argument of a `t(` or
/// `nt(` call.
fn preceded_by_translation_call(chars: &[char], i: usize) -> bool {
	let mut j = i;
	while j > 0 && chars[j - 1].is_whitespace() {
		j -= 1;
	}
	if j == 0 || chars[j - 1] != '(' {
		return false;
	}
	j -= 1;
	if j == 0 || chars[j - 1] != 't' {
		return false;
	}
	j -= 1;
	if j > 0 && chars[j - 1] == 'n' {
		j -= 1;
	}
	j == 0 || !(chars[j - 1].is_alphanumeric() || chars[j - 1] == '_')
}

/// The hash count of the raw string opening at `chars[i]` (`r"` is 0, `r#"` is
/// 1, ...), or `None` when `chars[i]` is an ordinary `r`.
fn raw_string_hash_count(chars: &[char], i: usize) -> Option<usize> {
	let n = chars.len();
	let mut j = i + 1;
	let mut hashes = 0usize;
	while j < n && chars[j] == '#' {
		hashes += 1;
		j += 1;
	}
	if j < n && chars[j] == '"' { Some(hashes) } else { None }
}

/// Blank the body of a raw string whose opening delimiter `i` sits just past,
/// up to and including the matching `"` plus `hashes` `#`s.
fn blank_raw_string_body(chars: &[char], i: &mut usize, hashes: usize, out: &mut String) {
	let n = chars.len();
	while *i < n {
		if chars[*i] == '"' {
			let mut k = *i + 1;
			let mut matched = 0usize;
			while k < n && matched < hashes && chars[k] == '#' {
				matched += 1;
				k += 1;
			}
			if matched == hashes {
				for _ in 0..=hashes {
					out.push(' ');
				}
				*i = k;
				return;
			}
		}
		if chars[*i] == '\n' {
			out.push('\n');
		} else {
			out.push(' ');
		}
		*i += 1;
	}
}

/// The index of the closing `'` when `chars[i]` opens a character literal
/// (`'x'`, `'\n'`, `'\''`, `'\u{2603}'`), or `None` when it opens a lifetime.
fn char_literal_end(chars: &[char], i: usize) -> Option<usize> {
	let n = chars.len();
	let mut j = i + 1;
	if j >= n {
		return None;
	}
	if chars[j] == '\\' {
		j += 1;
		if j >= n {
			return None;
		}
		if chars[j] == 'u' && j + 1 < n && chars[j + 1] == '{' {
			let mut k = j + 2;
			while k < n && chars[k] != '}' {
				k += 1;
			}
			if k >= n {
				return None;
			}
			j = k + 1;
		} else {
			j += 1;
		}
	} else {
		j += 1;
	}
	if j < n && chars[j] == '\'' { Some(j) } else { None }
}

#[cfg(test)]
mod tests {
	use super::*;

	fn sanitize(src: &str) -> String {
		sanitize_for_xgettext(src).text
	}

	#[test]
	fn preserves_normal_strings_and_comments() {
		let src = "// TRANSLATORS: hi\nlet x = t(\"hello 'world'\");\n";
		assert_eq!(sanitize(src), src);
	}

	#[test]
	fn neutralizes_lifetimes_without_changing_length_or_lines() {
		let src = "fn f<'a>(s: &'a str) -> &'a str { s }\n";
		let out = sanitize(src);
		assert_eq!(out.chars().count(), src.chars().count());
		assert_eq!(out.lines().count(), src.lines().count());
		assert!(!out.contains('\''));
	}

	#[test]
	fn neutralizes_static_and_anonymous_lifetimes() {
		let out = sanitize("const A: &'static str = \"x\"; fn g(v: Vec<'_>) {}");
		assert!(!out.contains('\''));
		assert!(out.contains("\"x\""));
	}

	#[test]
	fn preserves_char_literals() {
		let src = "let c = '{'; let d = '\\n'; let e = '\\'';\n";
		let out = sanitize(src);
		assert!(out.contains("'{'"));
		assert!(out.contains("'\\n'"));
		assert!(out.contains("'\\''"));
	}

	#[test]
	fn blanks_raw_strings_but_keeps_line_count() {
		let src = "let xml = r#\"\n<a href=\"x\">it's</a>\n\"#;\nlet y = t(\"real\");\n";
		let out = sanitize(src);
		assert_eq!(out.lines().count(), src.lines().count());
		assert!(out.contains("t(\"real\")"));
		assert!(!out.contains("href"));
	}

	#[test]
	fn blanks_backslash_continued_literals_but_keeps_line_count() {
		let src = concat!(
			"pub const CSS: &str = \"\\\n",
			"body { font-family:'SBL BibLit'; }\n",
			"a { color:#2a6bbf; }\";\n",
			"let y = t(\"real\");\n",
		);
		let out = sanitize(src);
		assert_eq!(out.lines().count(), src.lines().count());
		assert!(!out.contains("font-family"));
		assert!(!out.contains('\''));
		assert!(out.contains("t(\"real\")"));
	}

	#[test]
	fn reports_a_blanked_literal_that_was_inside_a_call() {
		let plain = sanitize_for_xgettext("let s = format!(\"a\\\nb\");\n");
		assert!(plain.blanked_call_literals.is_empty());
		let dropped = sanitize_for_xgettext("let a = 1;\nlet s = t(\"a\\\nb\");\n");
		assert_eq!(dropped.blanked_call_literals, vec![2]);
		let plural = sanitize_for_xgettext("let s = nt(\"a\\\nb\", \"c\", n);\n");
		assert_eq!(plural.blanked_call_literals, vec![1]);
	}

	#[test]
	fn keeps_translators_comment_adjacent_to_its_call() {
		let src = concat!(
			"let css = r#\"body { font: 'SBL BibLit'; }\"#;\n",
			"// TRANSLATORS: label\n",
			"let s = t(\"Ready\");\n",
		);
		let out = sanitize(src);
		let lines: Vec<&str> = out.lines().collect();
		assert!(lines[1].contains("TRANSLATORS"));
		assert!(lines[2].contains("t(\"Ready\")"));
	}

	#[test]
	fn leaves_identifiers_starting_with_r_alone() {
		let src = "let result = rows.len(); let r = 1;\n";
		assert_eq!(sanitize(src), src);
	}

	#[test]
	fn leaves_escaped_quotes_inside_single_line_literals_alone() {
		let src = "let s = t(\"say \\\"hi\\\" now\");\n";
		assert_eq!(sanitize(src), src);
	}
}
