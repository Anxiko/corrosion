use std::cmp::min;
use std::ops::Range;

#[derive(Debug, Clone, Eq, PartialEq)]
struct TileIndex {
	index: u8,
	bits: Option<Range<u8>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct TileIndexRange {
	bit_start: u16,
	bit_end: u16, // Exclusive
}

impl TileIndexRange {
	fn new(bit_start: u16, bit_end: u16) -> Self {
		Self { bit_start, bit_end }
	}
}

struct TileIndexIterator {
	position: u16,
	remaining: u16,
}

impl TileIndexIterator {
	fn new(position: u16, remaining: u16) -> Self {
		Self { position, remaining }
	}

	fn range(start: u16, end: u16) -> Self {
		Self::new(start, end - start)
	}
}

impl Iterator for TileIndexIterator {
	type Item = TileIndex;

	fn next(&mut self) -> Option<Self::Item> {
		if self.remaining == 0 {
			return None;
		}

		let index = self.position / 8;
		let bit_start = (self.position % 8) as u8;
		let length = min(8 - (bit_start as u16), self.remaining) as u8;
		let bit_end = bit_start + length;

		self.position += length as u16;
		self.remaining -= length as u16;

		let bits = match (bit_start, bit_end) {
			(0, 8) => None,
			_ => Some(bit_start..bit_end),
		};

		Some(TileIndex {
			index: index as u8,
			bits,
		})
	}
}

impl IntoIterator for TileIndexRange {
	type Item = TileIndex;
	type IntoIter = TileIndexIterator;

	fn into_iter(self) -> Self::IntoIter {
		Self::IntoIter::range(self.bit_start, self.bit_end)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn half_ends_range() {
		let start = 4 * 8 + 3;
		let end = start + 10 * 8;
		let range = TileIndexRange::new(start, end);

		let actual = range.into_iter().collect::<Vec<_>>();
		let mut expected = vec![TileIndex {
			index: 4,
			bits: Some(3..8),
		}];
		expected.extend((0..9).map(|x| x + 5).map(|x| TileIndex { index: x, bits: None }));
		expected.push(TileIndex {
			index: 4+10,
			bits: Some(0..3),
		});

		assert_eq!(actual, expected);
	}
}
