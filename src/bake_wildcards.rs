use std::rc::Rc;

use crate::{Database, Decl, GlobalIdent, WildcardImport};

impl Database {
	
	pub(crate) fn bake_wildcards(&mut self) {
		loop {
			let mut batch: Vec<(GlobalIdent, GlobalIdent, Rc<WildcardImport>)> = Default::default();
			for wildcard in self.wildcard_imports.iter() {
				let query = self.decls.range(wildcard.source.clone()..);
				for (ident, _decl)in query {
					if let Decl::Ast(_ast ) = _decl {
						if let Some(it) = ident.try_replace_base(&wildcard.source, &wildcard.target) {
							batch.push((it, ident.clone(), wildcard.clone()));
						}
					}
				}
			}
			let mut changes = 0;
			for (target, source, w) in batch {
				// println!("baked decl: {target}, {source}");
				if self.decls.insert(target, Decl::WildcardImport(source, w)).is_none() {
					changes += 1;
				}
			}
			println!("changes: {changes}");
			if changes == 0 {
				break;
			}
		}
		self.wildcard_imports.clear();
	}
}