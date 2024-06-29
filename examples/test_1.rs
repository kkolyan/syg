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

	lws.force_complete();

	let mut f = String::new();
	db.print_to(&mut f).unwrap();

	fs::write("examples/test_1.yaml", f).unwrap();

}
