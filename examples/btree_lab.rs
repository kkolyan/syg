use std::collections::BTreeMap;

fn main() {
	let mut map: BTreeMap<&str, i32> = Default::default();
	map.insert("abc", 1);
	map.insert("bcd", 2);
	map.insert("bcde", 3);
	map.insert("cde", 4);

	println!("all:");

	for (k, v) in map.iter() {
		println!("  {}: {}", k, v);
	}

	let r = map.range("bc"..);

	println!("range bc..z");

	for (k, v) in r {
		println!("  {}: {}", k, v);
	}
}