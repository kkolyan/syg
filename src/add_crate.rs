use core::fmt;
use std::{fs, rc::Rc, str::from_utf8, sync::Arc};

use quote::ToTokens;
use syn::{parse_file, visit::Visit, visit_mut::VisitMut, Expr, Ident, Item, Lit, MetaNameValue, UseTree};

use crate::{
    resolve_idents::BlocksClear, Database, Decl, DeclAst, GlobalIdent, IdentPart, ImportKind, RefstrExt, WildcardImport
};

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
}

struct SymbolsExplorer<'a> {
    crate_src: String,
    mod_stack: Vec<String>,
    db: &'a mut Database,
}

impl SymbolsExplorer<'_> {
    fn with_mod<T>(&mut self, name: &str, f: impl FnOnce(&mut SymbolsExplorer) -> T) -> T {
        let name = name.to_string().replace('-', "_");
        let parent_path = GlobalIdent::from_path(&self.mod_stack);
        let parent = self.db.decls.find_mut_unchecked(&parent_path);

        println!(
            "add mod {}",
            GlobalIdent::from_mod_and_name(&parent_path, name.as_str())
        );

        self.mod_stack.push(name.clone());

        parent.add_child(
            IdentPart::from_name(&name),
            Decl::Mod(crate::Mod {
                address: GlobalIdent::from_mod_and_name(&parent_path, &name),
                wildcard_imported_mods: Default::default(),
            }),
        );

        let r = f(self);
        self.mod_stack.pop();
        r
    }

    fn add_file(&mut self, name: &str, fs_path: &str) {
        println!("add_file( {:?}, {:?} )", name, fs_path);
        let content = fs::read(fs_path).unwrap();
        let content = from_utf8(&content).unwrap();
        let mut ast = parse_file(content).unwrap();
        BlocksClear.visit_file_mut(&mut ast);
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
                    if name == "self" {
                        self.mod_stack.clone()
                    } else {
                        if name == "crate" {
                            name.clone_from(self.mod_stack.first().unwrap());
                        }
                        vec![name]
                    }
                } else {
                    let mut v = path.clone();
                    v.push(it.ident.to_string());
                    v
                };
                self.collect_uses(&it.tree, new_path);
            }
            UseTree::Name(it) => {
                let taget = GlobalIdent::from_path_and_ident(&self.mod_stack, &it.ident);
                let source = GlobalIdent::from_path_and_name(&path, it.ident.to_string().as_str());
                println!("add import {} (from {})", taget, source);
                self.db
                    .decls
                    .find_mut_unchecked(&taget.parent())
                    .add_child(
                        taget.last_part(),
                        Decl::Import(
                            source,
                            ImportKind::Normal,
                        ),
                    );
            }
            UseTree::Rename(it) => {
                let source = GlobalIdent::from_path_and_name(&path, it.ident.to_string().as_str());
                let target = GlobalIdent::from_path_and_ident(&self.mod_stack, &it.rename);
                println!("add import {} (from {})", target, source);
                self.db
                    .decls
                    .find_mut_unchecked(&target.parent())
                    .add_child(
                        target.last_part(),
                        Decl::Import(
                            source,
                            ImportKind::Normal,
                        ),
                    );
            }
            UseTree::Glob(_it) => {
                let current_mod = self
                    .db
                    .decls
                    .find_mut_unchecked(&GlobalIdent::from_path(&self.mod_stack))
                    .get_value_mut();
                let current_mod = match current_mod {
                    Decl::Mod(it) => it,
                    _ => panic!("not a mod? WTF"),
                };
                current_mod
                    .wildcard_imported_mods
                    .insert(GlobalIdent::from_path(&path));

                self.db.wildcard_imports_temp.push(Rc::new(WildcardImport {
                    target: GlobalIdent::from_path(&self.mod_stack),
                    source: GlobalIdent::from_path(&path),
                }));
            }
            UseTree::Group(it) => {
                for it in it.items.iter() {
                    self.collect_uses(it, path.clone());
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for SymbolsExplorer<'_> {
    fn visit_item(&mut self, i: &'ast syn::Item) {
        syn::visit::visit_item(self, i);
        if let Item::Mod(_) = i {
            // mods are already handled
            return;
        }
        if let Some(ident) = Self::ident_of_item(i) {
            if ident == "test" {
                return;
            }
            if ident == "tests" {
                return;
            }
            let address = GlobalIdent::from_path_and_ident(&self.mod_stack, ident);
            let node = self.db.decls.find_mut_unchecked(&address.parent());
            println!("add ast {}", address);
            node.add_child(
                IdentPart::from_ident(ident),
                Decl::Ast(DeclAst {
                    address,
                    ast: i.clone(),
                }),
            );
        }
    }

    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        if i.ident == "test" {
            return;
        }
        if i.ident == "tests" {
            return;
        }
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

    fn visit_item_use(&mut self, i: &'ast syn::ItemUse) {
        self.collect_uses(&i.tree, vec![]);
    }
}
