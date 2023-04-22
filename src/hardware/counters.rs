pub(crate) mod divider;
pub(crate) mod timer;

trait Tick {
	fn tick(&mut self);
}
