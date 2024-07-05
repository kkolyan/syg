use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    rc::Rc,
};

use proc_macro2::Span;
use quote::quote;
use syn::{parse2, parse_str, Ident, Item, ItemStruct, Path};

use crate::GlobalIdent;

#[derive(Default, Debug)]
pub struct Database {
    pub decls: BTreeMap<GlobalIdent, Decl>,
    pub wildcard_imports: Vec<Rc<WildcardImport>>,
    pub unresolved: BTreeMap<String, UnresolvedCtx>,
}

#[derive(Debug, Default)]
pub struct UnresolvedCtx {
    pub scopes: BTreeSet<GlobalIdent>,
    pub requestors: BTreeSet<GlobalIdent>,
}

impl Database {
    pub fn compile(&mut self) {
        self.bake_wildcards();
        self.resolve_idents();
    }

    pub fn lookup_decl(&self, candidate: &GlobalIdent) -> Option<(GlobalIdent, &DeclAst)> {
        if let Some(it) = self.decls.get(candidate) {
            match it {
                Decl::Ast(it) => return Some((candidate.clone(), it)),
                Decl::Import(it) => {
                    if let Some(it) = self.lookup_decl(it) {
                        return Some(it);
                    }
                }
                Decl::WildcardImport(it, _w) => {
                    if let Some(it) = self.lookup_decl(it) {
                        return Some(it);
                    }
                }
            }
        }
        None
    }

    pub fn print_to(&self, f: &mut dyn fmt::Write) -> fmt::Result {
        writeln!(f, "decls:")?;
        for (ident, _decl) in self.decls.iter() {
            writeln!(f, "  - {}", ident)?;
        }
        writeln!(f, "use_wildcards:")?;
        for wildcard in self.wildcard_imports.iter() {
            writeln!(f, "  - {:?}: {:?}", wildcard.target, wildcard.source)?;
        }
        Ok(())
    }

    pub fn add_type_stub(&mut self, name: &str) {
		let path = parse_str::<Path>(name).unwrap();
		let ident = path.segments.last().unwrap().ident.clone();
		let ast = quote! { pub struct #ident {} };
		let ast = parse2::<Item>(ast.clone()).unwrap_or_else(|_| panic!("{}", ast));
        self.decls.insert(
            GlobalIdent::from_qualified_name(name),
            Decl::Ast(DeclAst::Ok(ast)),
        );
    }
}

#[derive(Debug)]
pub struct WildcardImport {
    pub target: GlobalIdent,
    pub source: GlobalIdent,
}

#[derive(Debug)]
pub enum Decl {
    Ast(DeclAst),
    Import(GlobalIdent),
    WildcardImport(GlobalIdent, Rc<WildcardImport>),
}

#[derive(Debug)]
pub enum DeclAst {
    Ok(Item),
    Borrowed,
}

impl DeclAst {
    pub fn unwrap(&self) -> &Item {
        match self {
            DeclAst::Ok(it) => it,
            DeclAst::Borrowed => panic!("this AST is borrowed"),
        }
    }
}
