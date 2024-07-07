use std::{
    borrow::BorrowMut,
    collections::{BTreeMap, BTreeSet},
    mem,
    rc::Rc,
};

use quote::ToTokens;
use syn::{visit::Visit, visit_mut::VisitMut, Item, Path, PathResolution};
use to_vec::ToVec;

use crate::{
    dedoc::{ItemExt, ItemTypeExt}, display_utils::DisplaySlice, ident_part::RefSliceOfIdentPartExt, named_tree::{FromPath, NamedNode}, stopwatch::start_watch, BindingResolution, Database, Decl, DeclAst, GlobalIdent, IdentPart, Mod, UnresolvedCtx, WildcardImport
};

#[derive(Debug, Default)]
struct Resolved {
    type_ast: Option<Item>,
    non_type_ast: Option<Item>,
}

impl<K> FromPath<K> for Resolved {
    fn from_path(_path: &[K]) -> Self {
        Default::default()
    }
}

impl Database {
    pub(crate) fn resolve_idents(&mut self) {
        let _ri = start_watch("resolve_idents");
        self.decls.for_each_mut(&mut |k, decl, path| {
            let ident = GlobalIdent::from_ident_path(k);
            if let Some(decl) = &decl.type_ast {
                println!(
                    "decl type {} -> {}",
                    ident,
                    decl.ast.as_ref().map(|it| it.dedoc().to_token_stream().to_string()).unwrap_or("<stubbed type>".to_owned())
                );
            }
            if let Some(decl) = &decl.non_type_ast {
                println!(
                    "decl expr {} -> {}",
                    ident,
                    decl.ast.as_ref().map(|it| it.dedoc().to_token_stream().to_string()).unwrap_or("<stubbed type>".to_owned())
                );
            }
            for (import, _) in decl.alias_for.iter() {
                println!("decl {} -> (use) {}", ident, import);
            }
        });
        let mut resolved: NamedNode<IdentPart, Resolved> = Default::default();
        let mut unresolved: BTreeMap<GlobalIdent, UnresolvedCtx> = Default::default();
        let mut resolutions: NamedNode<IdentPart, BindingResolution> = Default::default();

        self.decls.for_each(&mut |key, decl| {
            let key = GlobalIdent::from_ident_path(key);

            for decl in &decl.type_ast {
                println!("resolve decl {}", key);

                let mut ast = decl.ast.clone();

                if let Some(ast) = ast.as_mut() {
                    SymbolsResolve {
                        db: self,
                        parent: key.parent(),
                        key: key.clone(),
                        unresolved: &mut unresolved,
                        resolutions: &mut resolutions,
                    }
                    .visit_item_mut(ast);
                }

                resolved.find_or_create(&key).get_value_mut().type_ast = ast;
            }
            for decl in &decl.non_type_ast {
                println!("resolve decl {}", key);

                let mut ast = decl.ast.clone();
                if let Some(ast) = ast.as_mut() {
                    SymbolsResolve {
                        db: self,
                        parent: key.parent(),
                        key: key.clone(),
                        unresolved: &mut unresolved,
                        resolutions: &mut resolutions,
                    }
                    .visit_item_mut(ast);
                }

                resolved.find_or_create(&key).get_value_mut().non_type_ast = ast;
            }
        });

        self.decls
            .left_join(Some(&resolved), &mut |decls, resolved| {
                if let Some(ast) = &mut decls.non_type_ast {
                    if let Some(Resolved {
                        non_type_ast: Some(resolved),
                        ..
                    }) = resolved
                    {
                        ast.ast = Some(resolved.clone());
                    }
                }
                if let Some(ast) = &mut decls.type_ast {
                    if let Some(Resolved {
                        type_ast: Some(resolved),
                        ..
                    }) = resolved
                    {
                        ast.ast = Some(resolved.clone());
                    }
                }
            });

            self.decls
                .left_join(Some(&resolutions), &mut |decls, resolution| {
                    if let Some(resolution) = resolution {
                        decls.resolution.and(*resolution);
                    }
                });

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

pub struct BlocksClear;

impl VisitMut for BlocksClear {
    fn visit_block_mut(&mut self, i: &mut syn::Block) {
        i.stmts.clear();
    }
}

struct SymbolsResolve<'a> {
    db: &'a Database,
    parent: GlobalIdent,
    key: GlobalIdent,
    unresolved: &'a mut BTreeMap<GlobalIdent, UnresolvedCtx>,
    resolutions: &'a mut NamedNode<IdentPart, BindingResolution>,
}

impl VisitMut for SymbolsResolve<'_> {
    fn visit_attribute_mut(&mut self, _i: &mut syn::Attribute) {}

    fn visit_visibility_mut(&mut self, _i: &mut syn::Visibility) {}

    fn visit_expr_mut(&mut self, _i: &mut syn::Expr) {}

    fn visit_path_mut(&mut self, i: &mut syn::Path) {
        println!("  visit {}", i.to_token_stream());
        let path = i.segments.iter().map(|it| it.ident.to_string()).to_vec();
        if path.len() == 1 && path[0] == "Self" {
            return;
        }
        let candidates = [
            GlobalIdent::from_mod_and_path(&self.parent, &path),
            GlobalIdent::from_path(&path),
        ];

        let mut partial_resolutions: BTreeSet<GlobalIdent> = Default::default();

        let mut binding_resolution = BindingResolution::NotAttempted;

        for candidate in candidates.iter() {
            println!("    candidate {}", candidate);
            if *candidate == self.key {
                let mut res = i.clone();
                self.key.qualify_syn_path(&mut res);
                i.resolution = PathResolution::Resolved(res.clone().into());
                println!("      resolved to {} ({:?})", res.to_token_stream(), res);
                self.resolutions.find_or_create(&self.key).get_value_mut().and(BindingResolution::Fully);
                return;
            }

            let resolution = self.db.lookup_decl(candidate);
            match resolution {
                crate::Resolution::Fully(DeclAst { address, ast }) => {
                    println!("      resolved to address {}", address);
                    if let Some(Item::Type(decl)) = ast {
                        println!("      type {}", decl.dedoc().to_token_stream());
                    } else {
                        println!("      ast {}", ast.to_token_stream());
                    }
                    let mut res = i.clone();
                    address.qualify_syn_path(&mut res);
                    i.resolution = PathResolution::Resolved(res.clone().into());
                    println!("      resolved to {} ({:?})", res.to_token_stream(), res);
                    self.resolutions.find_or_create(&self.key).get_value_mut().and(BindingResolution::Fully);
                    return;
                }
                crate::Resolution::Partially(it) => {
                    partial_resolutions.insert(it.clone());
                    binding_resolution.or(BindingResolution::Partially);
                }
                crate::Resolution::Failed => {},
            }
        }
        println!("    unresolved");
        i.resolution = PathResolution::Failed;
        binding_resolution.or(BindingResolution::Failed);
        self.resolutions.find_or_create(&self.key).get_value_mut().and(binding_resolution);

        println!("    partial resolutions:");
        for it in partial_resolutions.iter() {
            println!("        {}", it);
        }

        if partial_resolutions.len() > 1 {
            panic!("multiple partial resolutions: {:?}", partial_resolutions);
        }
        if partial_resolutions.is_empty() {
            println!("    no resolutions found for {}", DisplaySlice(&candidates));
            partial_resolutions = BTreeSet::from(candidates);
        }

        let ctx = self
            .unresolved
            .entry(partial_resolutions.into_iter().next().unwrap())
            .or_default();

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
