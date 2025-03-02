use super::fontstack::Fontstack;
use prost::{alloc, Message};

#[derive(Clone, PartialEq, Message)]
pub struct Glyphs {
	#[prost(message, repeated, tag = "1")]
	pub stacks: alloc::vec::Vec<Fontstack>,
}
