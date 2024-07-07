use std::collections::BTreeMap;

use quote::ToTokens;
use syn::{visit::Visit, Item, ItemType, Path, Type, TypePath};

use crate::{
    check_path_resolved::PathResolutionCheck, dedoc::ItemTypeExt, ident_part::RefSliceOfIdentPartExt, stopwatch::start_watch, Ast, Database, GlobalIdent
};

impl Database {
    pub(crate) fn inline_types(&mut self) {
        let _watch = start_watch("inline_types");
        /*
        traverse decls
            - to collect all type aliases to external collection
        traverse over this collection
            - write alias name to the final type decl (to know all aliases)
        traverse decls
            - traverse ast
                - write final type everywhere (replace original Path with final type AST,
                which could be not a path, though, I'm not sure we really should support this non-Path case)
         */

        let mut type_by_alias: BTreeMap<GlobalIdent, Path> = Default::default();

        self.decls.for_each(&mut |path, binding| {
			if let Some(ast) = &binding.type_ast {
				if let Ast::Real(ast) = &ast.ast {
					if let Item::Type(ast) = ast {
						match &*ast.ty {
							Type::Path(ty) => {
								if ty.qself.is_some() {
									println!("WARN: qself is not implemented. {} for {}", ty.to_token_stream(), path.to_global_path())
								} else {
									let check = PathResolutionCheck::check_path(&ty.path);
									if check.not_attempted.node_count() > 1 || check.failed.node_count() > 1 {
										println!("WARN: resolution not attempted or failed for the type of {}: {}", path.to_global_path(), ast.dedoc().to_token_stream());
										check.aggregated.for_each(&mut |path, v| {
											for (ty, res) in v.iter() {
												let res = match res {
													syn::PathResolution::NotAttempted => "NotAttempted".to_owned(),
													syn::PathResolution::Failed => "Failed".to_owned(),
													syn::PathResolution::Resolved(it) => format!("Resolved({})", it.to_token_stream()),
												};
												println!("{}{} - {}", "    ".repeat(path.len()), ty, res);	
											}
										});
									} else {
										println!("INFO: resolution OK for {}: {}", path.to_global_path(), ast.dedoc().to_token_stream());
										type_by_alias.insert(path.to_global_path(), ty.path.clone());
									}
								}
							},
							_ => {
								println!("WARN: type alias ignored, because non-path aliases not supported: {} ({:?})", ast.dedoc().to_token_stream(), ast.dedoc());
							},
						}
					}
				}
			}
		 });

        for (path, type_) in type_by_alias.iter() {
            // self.lookup_decl(candidate)
        }
    }
}
