//! Session history of visited lemmas: a most-recent-first list of
//! dictionary word indices, deduplicated and capped.

pub const CAPACITY: usize = 20;

#[derive(Debug, Default)]
pub struct History {
	entries: Vec<usize>,
}

impl History {
	/// Move `idx` to the front, inserting it when absent, and drop entries
	/// past `CAPACITY`; returns false when `idx` already was the front entry.
	pub fn push(&mut self, idx: usize) -> bool {
		if self.entries.first() == Some(&idx) {
			return false;
		}
		self.entries.retain(|&e| e != idx);
		self.entries.insert(0, idx);
		self.entries.truncate(CAPACITY);
		true
	}

	/// The visited word indices, most recent first.
	pub fn entries(&self) -> &[usize] {
		&self.entries
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn push_puts_the_entry_in_front() {
		let mut history = History::default();
		assert!(history.push(3));
		assert!(history.push(7));
		assert_eq!(history.entries(), &[7, 3]);
	}

	#[test]
	fn re_pushing_the_front_entry_changes_nothing() {
		let mut history = History::default();
		history.push(3);
		history.push(7);
		assert!(!history.push(7));
		assert_eq!(history.entries(), &[7, 3]);
	}

	#[test]
	fn re_pushing_a_deeper_entry_promotes_it_without_duplicating() {
		let mut history = History::default();
		history.push(3);
		history.push(7);
		history.push(9);
		assert!(history.push(3));
		assert_eq!(history.entries(), &[3, 9, 7]);
	}

	#[test]
	fn the_oldest_entry_drops_past_the_capacity() {
		let mut history = History::default();
		for i in 0..=CAPACITY {
			history.push(i);
		}
		assert_eq!(history.entries().len(), CAPACITY);
		assert_eq!(history.entries()[0], CAPACITY);
		assert!(!history.entries().contains(&0));
	}

	#[test]
	fn promoting_an_existing_entry_never_grows_past_the_capacity() {
		let mut history = History::default();
		for i in 0..CAPACITY {
			history.push(i);
		}
		assert!(history.push(0));
		assert_eq!(history.entries().len(), CAPACITY);
		assert_eq!(history.entries()[0], 0);
		assert!(history.entries().contains(&1));
	}
}
