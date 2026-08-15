"""Fail when a TRANSLATORS comment will not reach the pot file.

`patois_build::gen_pot` extracts with
`xgettext --keyword=t --language=C --add-comments=TRANSLATORS`, and xgettext
keeps a comment block only when the line right after it holds the `t(` call.
rustfmt wrapping a long statement moves `t(` down a line and silently drops the
comment, so this checks that every block is still adjacent to its call.

Takes the files to check as arguments; without any, walks the crate sources.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE_DIRS = ("crates/dictymus/src", "crates/dictymus-core/src", "crates/dictymus-container/src")
COMMENT = "// TRANSLATORS"
CALL = re.compile(r"\bt\(")


def orphaned(path):
	"""Line numbers of TRANSLATORS blocks in `path` not followed by a t( call."""
	try:
		lines = path.read_text(encoding="utf-8").splitlines()
	except (OSError, UnicodeDecodeError):
		return []
	found = []
	for i, line in enumerate(lines):
		if COMMENT not in line:
			continue
		# Skip the rest of a multi-line comment block to reach the code line.
		j = i + 1
		while j < len(lines) and lines[j].lstrip().startswith("//"):
			j += 1
		if j >= len(lines) or not CALL.search(lines[j]):
			found.append((i + 1, line.strip()))
	return found


def main(argv):
	if argv:
		paths = [Path(a) for a in argv if a.endswith(".rs")]
	else:
		paths = [p for d in SOURCE_DIRS for p in sorted((ROOT / d).rglob("*.rs"))]

	failures = [(path, line, text) for path in paths for line, text in orphaned(path)]
	for path, line, text in failures:
		print(f"{path}:{line}: TRANSLATORS comment is not on the line above its t() call")
		print(f"    {text}")
	if failures:
		print("\nxgettext drops these. Move the comment onto the t() line, or shorten")
		print("the statement so rustfmt keeps t( on the line below the comment.")
		return 1
	return 0


if __name__ == "__main__":
	sys.exit(main(sys.argv[1:]))
