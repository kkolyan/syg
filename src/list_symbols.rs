use core::fmt;
use std::{collections::BTreeMap, fs, str::from_utf8};

use quote::ToTokens;
use syn::{
    fold::Fold, parse_file, visit::Visit, Expr, Ident, Item, Lit, LitStr, MetaNameValue, PatLit,
    UseTree,
};

use crate::{
    model::{Database, GlobalIdent},
    RefstrExt,
};

struct SymbolsExplorer<'a> {
    crate_src: String,
    mod_stack: Vec<String>,
    db: &'a mut Database,
}

#[derive(Debug)]
pub struct UseWildcard {
    from: String,
    to: String,
}

#[derive(Debug)]
pub enum Decl {
    AST(syn::Item),
}

impl SymbolsExplorer<'_> {
    fn with_mod<T>(&mut self, name: &str, f: impl FnOnce(&mut SymbolsExplorer) -> T) -> T {
        self.mod_stack.push(name.to_string().replace('-', "_"));
        let r = f(self);
        self.mod_stack.pop();
        r
    }

    fn add_file(&mut self, name: &str, fs_path: &str) {
        println!("add_file( {:?}, {:?} )", name, fs_path);
        let content = fs::read(fs_path).unwrap();
        let content = from_utf8(&content).unwrap();
        let ast = parse_file(content).unwrap();
        self.with_mod(name, |visitor| {
            visitor.visit_file(&ast);
        });
    }

    fn ident_of_item(item: &Item) -> Option<&Ident> {
        match item {
            Item::Const(it) => Some(&it.ident),
            Item::Enum(it) => Some(&it.ident),
            Item::ExternCrate(it) => Some(&it.ident),
            Item::Fn(it) => Some(&it.sig.ident),
            Item::ForeignMod(_it) => None,
            Item::Impl(_it) => None,
            Item::Macro(_it) => None,
            Item::Mod(it) => Some(&it.ident),
            Item::Static(it) => Some(&it.ident),
            Item::Struct(it) => Some(&it.ident),
            Item::Trait(it) => Some(&it.ident),
            Item::TraitAlias(it) => Some(&it.ident),
            Item::Type(it) => Some(&it.ident),
            Item::Union(it) => Some(&it.ident),
            Item::Use(_it) => None,
            Item::Verbatim(it) => unimplemented!("Vermatim({})", it),
            it => unimplemented!("unknown: {:?}: {}", it, it.to_token_stream()),
        }
    }

    fn collect_uses(&mut self, tree: &UseTree, path: Vec<String>) {
        match tree {
            UseTree::Path(it) => {
                let new_path = if path.is_empty() {
                    let mut name = it.ident.to_string();
                    if name == "crate" {
						name.clone_from(self.mod_stack.first().unwrap());
					}
                    vec![name]
                } else {
                    let mut v = path.clone();
                    v.push(it.ident.to_string());
                    v
                };
                self.collect_uses(&it.tree, new_path);
            }
            UseTree::Name(it) => {
                let name = it.ident.to_string();
                self.db.use_aliases.insert(
                    GlobalIdent::from_path_and_name(&self.mod_stack, &name),
                    GlobalIdent::from_path_and_name(&path, &name),
                );
            }
            UseTree::Rename(it) => {
                self.db.use_aliases.insert(
                    GlobalIdent::from_path_and_name(
                        &self.mod_stack,
                        it.rename.to_string().as_str(),
                    ),
                    GlobalIdent::from_path_and_name(&path, it.ident.to_string().as_str()),
                );
            }
            UseTree::Glob(_it) => {
                self.db.use_wildcards.push(UseWildcard {
                    from: self.mod_stack.join("::") + "::",
                    to: path.join("::") + "::",
                });
            }
            UseTree::Group(it) => {
                for it in it.items.iter() {
                    self.collect_uses(it, path.clone());
                }
            }
        }
    }
}

impl Database {
    pub fn add_crate(&mut self, base_path: &str, name: &str) {
        let src_path = base_path.concat("/").concat(name).concat("/src");
        let lib_path = src_path.add_file_segment("lib.rs");
        let mut visitor = SymbolsExplorer {
            crate_src: src_path.to_string(),
            mod_stack: Default::default(),
            db: self,
        };
        visitor.add_file(name, &lib_path);
    }

    pub fn print_to(&self, f: &mut dyn fmt::Write) -> fmt::Result {
        writeln!(f, "decls:")?;
        for (ident, _decl) in self.decls.iter() {
            writeln!(f, "  - {}", ident)?;
        }
        writeln!(f, "use_aliases:")?;
        for (k, v) in self.use_aliases.iter() {
            writeln!(f, "  - {}: {}", k, v)?;
        }
        writeln!(f, "use_wildcards:")?;
        for UseWildcard { from, to } in self.use_wildcards.iter() {
            writeln!(f, "  - {:?}: {:?}", from, to)?;
        }
        Ok(())
    }
}

impl<'ast> Visit<'ast> for SymbolsExplorer<'_> {
    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        // println!("mod {}", i.ident);
        match &i.content {
            Some((_brace, content)) => {
                self.with_mod(i.ident.to_string().as_str(), |self_| {
                    for item in content {
                        self_.visit_item(item);
                    }
                });
            }
            None => {
                let mut fs_path = self.crate_src.clone();
                for item in self
                    .mod_stack
                    .iter()
                    // skip crate name
                    .skip(1)
                {
                    fs_path += "/";
                    fs_path += item;
                }
                fs_path += "/";

                let explicit_path = i.attrs.iter().find_map(|it| match &it.meta {
                    syn::Meta::Path(_) => None,
                    syn::Meta::List(_) => None,
                    syn::Meta::NameValue(MetaNameValue { path, value, .. }) => {
                        if path.to_token_stream().to_string() == "path" {
                            match value {
                                Expr::Lit(syn::ExprLit {
                                    lit: Lit::Str(s), ..
                                }) => Some(fs_path.concat(s.value())),
                                err => {
                                    panic!("unexpected #[path] value: {}", err.to_token_stream())
                                }
                            }
                        } else {
                            None
                        }
                    }
                });

                let dir_based_path = fs_path.concat(i.ident.to_string()).concat("/mod.rs");
                let file_based_path = fs_path.concat(i.ident.to_string()).concat(".rs");
                let file_path = if let Some(explicit_path) = explicit_path {
                    explicit_path
                } else if fs::metadata(&dir_based_path).is_ok() {
                    dir_based_path
                } else {
                    file_based_path
                };

                self.add_file(i.ident.to_string().as_str(), &file_path)
            }
        }
    }

    fn visit_item(&mut self, i: &'ast syn::Item) {
        if let Some(ident) = Self::ident_of_item(i) {
            self.db.decls.insert(
                GlobalIdent::from_path_and_name(&self.mod_stack, ident.to_string().as_str()),
                Decl::AST(i.clone()),
            );
        }
        if let Item::Use(i) = i {
            self.collect_uses(&i.tree, vec![]);
        }
        syn::visit::visit_item(self, i);
    }

    fn visit_item_use(&mut self, i: &'ast syn::ItemUse) {}

    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {}

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {}

    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {}

    fn visit_item_macro(&mut self, i: &'ast syn::ItemMacro) {}

    fn visit_item_static(&mut self, i: &'ast syn::ItemStatic) {}

    fn visit_item_extern_crate(&mut self, i: &'ast syn::ItemExternCrate) {}

    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {}

    fn visit_item_type(&mut self, i: &'ast syn::ItemType) {}

    fn visit_item_union(&mut self, i: &'ast syn::ItemUnion) {}

    fn visit_item_const(&mut self, i: &'ast syn::ItemConst) {}

    fn visit_item_foreign_mod(&mut self, i: &'ast syn::ItemForeignMod) {}

    fn visit_item_trait(&mut self, i: &'ast syn::ItemTrait) {}

    fn visit_item_trait_alias(&mut self, i: &'ast syn::ItemTraitAlias) {}
}
