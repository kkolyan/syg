use std::rc::Rc;

use quote::ToTokens;
use syn::{visit_mut::VisitMut, Item, PathResolution};
use to_vec::ToVec;

use crate::{
    dedoc::ItemExt, stopwatch::start_watch, Database, Decl, DeclAst, GlobalIdent, WildcardImport,
};

impl Database {
    pub(crate) fn resolve_idents(&mut self) {
        let _ri = start_watch("resolve_idents");
        for (ident, decl) in self.decls.iter() {
            if let Decl::Ast(decl) = decl {
                println!(
                    "decl {} -> {}",
                    ident,
                    decl.unwrap().dedoc().to_token_stream()
                );
            }
            if let Decl::Import(import) = decl {
                println!(
                    "decl {} -> (use) {}",
                    ident,
                    import,
                );
            }
        }
        let keys = self.decls.keys().cloned().collect::<Vec<_>>();
        for key in keys {
            let mut decl = self
                .decls
                .insert(key.clone(), Decl::Ast(DeclAst::Borrowed))
                .unwrap();
            match &mut decl {
                Decl::Ast(ast) => {
                    println!("resolve decl {}", key);
                    match ast {
                        DeclAst::Ok(ast) => {
                            BlocksClear.visit_item_mut(ast);
                            SymbolsResolve {
                                db: self,
                                parent: key.parent(),
                                key: key.clone(),
                            }
                            .visit_item_mut(ast);
                        }
                        DeclAst::Borrowed => unimplemented!("WTF"),
                    }
                }
                Decl::Import(_ident) => {}
                Decl::WildcardImport(_ident, _w) => {}
            }
            self.decls.insert(key, decl);
        }

        for (ident, ctx) in self.unresolved.iter() {
            println!("unresolved: {ident}");
            for it in ctx.scopes.iter() {
                // println!("  scope {it}");
            }
            for it in ctx.requestors.iter() {
                // println!("  requestor {it}");
            }
        }
    }
}

struct BlocksClear;

impl VisitMut for BlocksClear {
    fn visit_block_mut(&mut self, i: &mut syn::Block) {
        i.stmts.clear();
    }
}

struct SymbolsResolve<'a> {
    db: &'a mut Database,
    parent: GlobalIdent,
    key: GlobalIdent,
}

impl VisitMut for SymbolsResolve<'_> {
    fn visit_attribute_mut(&mut self, i: &mut syn::Attribute) {}

    fn visit_visibility_mut(&mut self, i: &mut syn::Visibility) {
    }

    fn visit_expr_mut(&mut self, i: &mut syn::Expr) {
    }

    fn visit_path_mut(&mut self, i: &mut syn::Path) {
        println!("  visit {}", i.to_token_stream());
        let path = i.segments.iter().map(|it| it.ident.to_string()).to_vec();
        let candidates = [
            GlobalIdent::from_mod_and_path(&self.parent, &path),
            GlobalIdent::from_path(&path),
        ];

        for candidate in candidates {
            println!("    candidate {}", candidate);
            if candidate == self.key {
                let mut res = i.clone();
                self.key.qualify_syn_path(&mut res);
                i.resolution = PathResolution::Resolved(res.clone().into());
                println!("      resolved to {}", res.to_token_stream());
                return;
            }

            if let Some((ident, decl)) = self.db.lookup_decl(&candidate) {
                let decl = match decl {
                    DeclAst::Ok(it) => it,
                    DeclAst::Borrowed => panic!(
                        "WTF? self-refering should be handled above. i: {}, parent: {}",
                        i.to_token_stream(),
                        self.parent
                    ),
                };
                if let Item::Type(decl) = decl {
                    println!("      type {}", decl.to_token_stream());
                } else {
                    let mut res = i.clone();
                    ident.qualify_syn_path(&mut res);
                    i.resolution = PathResolution::Resolved(res.clone().into());
                    println!("      resolved to {}", res.to_token_stream());
                    return;
                }
            }
        }
        println!("    unresolved");
        i.resolution = PathResolution::Failed;

        let nearest_ident = GlobalIdent::from_mod_and_path(&self.parent, &path);
        println!("      nearest_ident: {}", &nearest_ident);
        let nearest_resolution_candidate = match self.db.decls.get(&nearest_ident) {
            Some(decl) => {
                match decl {
                    Decl::Ast(_ast) => nearest_ident.to_string(),
                    Decl::Import(ident) => ident.to_string(),
                    Decl::WildcardImport(ident, _) => ident.to_string(),
                }
            },
            None => path.join("::"),
        };
        println!("      nearest_resolution_candidate: {}", &nearest_resolution_candidate);
        
        let ctx = self.db.unresolved.entry(nearest_resolution_candidate).or_default();

        ctx.scopes.insert(self.parent.clone());
        ctx.requestors.insert(self.key.clone());
    }
}

/*
   fn visit_item_enum_mut(&mut self, _i: &mut syn::ItemEnum) {}

   fn visit_item_impl_mut(&mut self, _i: &mut syn::ItemImpl) {}

   fn visit_item_fn_mut(&mut self, _i: &mut syn::ItemFn) {}

   fn visit_item_macro_mut(&mut self, _i: &mut syn::ItemMacro) {}

   fn visit_item_static_mut(&mut self, _i: &mut syn::ItemStatic) {}

   fn visit_item_extern_crate_mut(&mut self, _i: &mut syn::ItemExternCrate) {}

   fn visit_item_struct_mut(&mut self, _i: &mut syn::ItemStruct) {}

   fn visit_item_type_mut(&mut self, _i: &mut syn::ItemType) {}

   fn visit_item_union_mut(&mut self, _i: &mut syn::ItemUnion) {}

   fn visit_item_const_mut(&mut self, _i: &mut syn::ItemConst) {}

   fn visit_item_foreign_mod_mut(&mut self, _i: &mut syn::ItemForeignMod) {}

   fn visit_item_trait_mut(&mut self, _i: &mut syn::ItemTrait) {}

   fn visit_item_trait_alias_mut(&mut self, _i: &mut syn::ItemTraitAlias) {}
*/
