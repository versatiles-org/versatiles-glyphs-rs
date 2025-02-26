extern crate prost_build;

fn main() {
	prost_build::compile_protos(&["src/lib/protobuf/glyphs.proto"], &["src/proto/"]).unwrap();
}
