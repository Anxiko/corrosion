use dyn_partial_eq::{dyn_partial_eq, DynPartialEq};

#[dyn_partial_eq]
trait MyTrait {}

#[derive(DynPartialEq, PartialEq)]
struct MyA {
	i: i32,
}

impl MyTrait for MyA {}

#[derive(DynPartialEq, PartialEq)]
struct MyB {
	s: String,
}

impl MyTrait for MyB {}

fn main() {
	let boxed_a: Box<dyn MyTrait> = Box::new(MyA { i: 0 });
	let other_boxed_a: Box<dyn MyTrait> = Box::new(MyA { i: 0 });
	let boxed_b: Box<dyn MyTrait> = Box::new(MyB {
		s: "Hello".to_owned(),
	});
	let other_boxed_b: Box<dyn MyTrait> = Box::new(MyB {
		s: "World".to_owned(),
	});

	assert!(boxed_a == other_boxed_a);
	assert!(boxed_b != boxed_a);
}
