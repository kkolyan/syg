use std::fs;

use syg::{model::Database, stopwatch::start_watch};

fn main() {
    let lws = start_watch("parse syn all");

    let mut db = Database::default();
    db.add_crate("c:/dev/rust/fyrox_lua/engine", "fyrox-core");
    db.add_crate("c:/dev/rust/fyrox_lua/engine", "fyrox-impl");
    db.add_crate("c:/dev/rust/fyrox_lua/engine", "fyrox-graph");
    db.add_crate("c:/dev/rust/fyrox_lua/engine", "fyrox-math");
    db.add_crate("c:/dev/rust/fyrox_lua/engine", "fyrox-resource");
    db.add_crate("c:/dev/rust/fyrox_lua/engine", "fyrox-scripts");
    db.add_crate("c:/dev/rust/fyrox_lua/engine", "fyrox-sound");
    db.add_crate("c:/dev/rust/fyrox_lua/engine", "fyrox-ui");
    db.add_crate("c:/dev/rust", "nalgebra");
	db.add_type_stub("usize");
	db.add_type_stub("u8");
	db.add_type_stub("u16");
	db.add_type_stub("u32");
	db.add_type_stub("u64");
	db.add_type_stub("u128");
	db.add_type_stub("isize");
	db.add_type_stub("i8");
	db.add_type_stub("i16");
	db.add_type_stub("i32");
	db.add_type_stub("i64");
	db.add_type_stub("i128");
	db.add_type_stub("f16");
	db.add_type_stub("f32");
	db.add_type_stub("f64");
	db.add_type_stub("bool");
	db.add_type_stub("str");
	db.add_type_stub("std::result::Result");
	db.add_type_stub("std::io::Error");
	db.add_type_stub("std::fs::File");
	db.add_type_stub("std::time::Instant");
	db.add_type_stub("mpsc::Sender");
	db.add_type_stub("std::vec::Vec");
	db.add_type_stub("std::rc::Weak");
	db.add_type_stub("std::rc::Rc");

    lws.force_complete();

    let ri = start_watch("compile");
	db.compile();
	ri.force_complete();

    let mut f = String::new();
    db.print_to(&mut f).unwrap();

    fs::write("examples/test_1.yaml", f).unwrap();
}
