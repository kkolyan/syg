use std::rc::Rc;

use crate::{ident_part::RefSliceOfIdentPartExt, Database, Decl, GlobalIdent, ImportKind, WildcardImport};

impl Database {
	
	pub(crate) fn bake_wildcards(&mut self) {
		// loop {
		// 	let mut batch: Vec<(GlobalIdent, GlobalIdent, Rc<WildcardImport>)> = Default::default();
		// 	for wildcard in self.wildcard_imports_temp.iter() {
		// 		if let Some(source) = self.decls.find_mut(&wildcard.source) {
		// 			for child in source.children() {
		// 				batch.push((wildcard.target.clone(), child.path().to_global_path(), wildcard.clone()));
		// 			}
		// 		}
		// 	}
		// 	let mut changes = 0;
		// 	for (target, source, w) in batch {
		// 		// println!("baked decl: {target}, {source}");
		// 		let target = self.decls.find_mut(&target).unwrap();
		// 		if target.get_child(&source.last_part()).is_none() {
		// 			target.add_child(source.last_part(), Decl::Import(source, ImportKind::Wildcard));
		// 			changes += 1;
		// 		}
		// 	}
		// 	println!("changes: {changes}");
		// 	if changes == 0 {
		// 		break;
		// 	}
		// }
		self.wildcard_imports_temp.clear();
	}
}