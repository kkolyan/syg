use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashSet},
    default,
    fmt::{self, Display},
    rc::Rc,
};

use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse2, parse_str, Ident, Item, ItemStruct, Path};

use crate::{
    dedoc::ItemExt, ident_part::RefSliceOfIdentPartExt, named_tree::{FromPath, NamedNode}, Ast, Binding, Database, DeclAst, GlobalIdent, IdentPart, Resolution
};

impl Database {
	

    pub fn lookup_decl(&self, candidate: &GlobalIdent) -> Resolution {
        println!("      lookup_decl {}", candidate);

        let path = candidate.to_parts();
        self.lookup_internal(&self.decls, &path, 0, &mut Default::default())
    }

    fn lookup_internal<'a, 'b, 'c>(
        &'a self,
        base: &'b NamedNode<IdentPart, Binding>,
        path: &[IdentPart],
        depth: usize,
        checked: &mut HashSet<GlobalIdent>,
    ) -> Resolution
    where
        'a: 'c,
        'b: 'c,
    {
        if base.path().is_empty() && !checked.insert(path.to_global_path()) {
            return Resolution::Partially(base.path().to_global_path());
        }
        let indent = "  ".repeat(depth);
        println!(
            "      {}lookup \"{}\" against \"{}\" ({})",
            indent,
            path.to_global_path(),
            base.path().to_global_path(),
            base.get_value()
        );
        let value = base.get_value();

        for (import, _kind) in value.alias_for.iter() {
            println!("      {}import {}", indent, import);
            let mut new_path = import.to_parts();
            new_path.extend_from_slice(path);
            if let Resolution::Fully(result) =
                self.lookup_internal(&self.decls, &new_path, depth + 1, checked)
            {
                return Resolution::Fully(result);
            }
        }
        if path.is_empty() {
            if let Some(ast) = &value.type_ast {
                return Resolution::Fully(ast.clone());
            }
            if !value.alias_for.is_empty() {
                for (it, _) in value.alias_for.iter() {
                    if it.first_part() == "std" || it.first_part() == "core" {
                        return Resolution::Fully(DeclAst {
                            address: base.path().to_global_path(),
                            ast: Ast::Stub,
                        });
                    }
                }
                assert!(value.alias_for.len() == 1, "cannot choose partial resolution: {}", value);
                println!(
                    "      {}partial resolution by alias \"{}\"",
                    indent,
                    value.alias_for.first().unwrap().0.clone()
                );
                return Resolution::Partially(value.alias_for.first().unwrap().0.clone());
            }
			return Resolution::Failed;
		}
        println!(
            "      {}checking as a mod \"{}\"",
            indent,
            base.path().to_global_path()
        );
        let (first, rem) = path.split_first().unwrap();
        if let Some(decl) = base.get_child(first) {
            let mut new_base_path = base.path().to_vec();
            new_base_path.push(first.clone());
            return self.lookup_internal(decl, rem, depth + 1, checked);
        }
        if value.wildcard_alias_for.is_empty() {
            println!("      {}no wildcard imports", indent);
        }
        for wildcard_import in value.wildcard_alias_for.iter() {
            println!(
                "      {}checking wildcard import {} (path: {}, base_path: {})",
                indent,
                wildcard_import,
                path.to_global_path(),
                base.path().to_global_path()
            );

            let mut new_path = wildcard_import.to_parts();
            new_path.extend_from_slice(path);
            if let Resolution::Fully(result) =
                self.lookup_internal(&self.decls, &new_path, depth + 1, checked)
            {
                return Resolution::Fully(result);
            }
        }
        println!("      {}failed resolution by default", indent);
        Resolution::Failed
    }
}