pub(super) struct Texture<const W: usize, const H: usize> {
	pixels: [[u8; W]; H],
}

impl<const W: usize, const H: usize> Texture<W, H> {
	pub(super) fn new() -> Self {
		Self { pixels: [[0; W]; H] }
	}
}