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
    dedoc::ItemExt, ident_part::RefSliceOfIdentPartExt, named_tree::{FromPath, NamedNode}, GlobalIdent,
    IdentPart,
};

#[derive(Debug)]
pub struct Database {
    pub decls: NamedNode<IdentPart, Binding>,
    pub wildcard_imports_temp: Vec<Rc<WildcardImport>>,
    pub unresolved: BTreeMap<String, UnresolvedCtx>,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            decls: NamedNode::new(Binding::new_empty(GlobalIdent::root())),
            wildcard_imports_temp: Default::default(),
            unresolved: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct UnresolvedCtx {
    pub scopes: BTreeSet<GlobalIdent>,
    pub requestors: BTreeSet<GlobalIdent>,
}

impl Database {
    pub fn compile(&mut self) {
        // TODO delete it
        // self.bake_wildcards();
        self.resolve_idents();
    }

    pub fn lookup_decl(&self, candidate: &GlobalIdent) -> Option<&DeclAst> {
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
    ) -> Option<&'c DeclAst>
    where
        'a: 'c,
        'b: 'c,
    {
        if base.path().is_empty() && !checked.insert(path.to_global_path()) {
            return None;
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
		if let Some(ast) = &value.type_ast {
			if path.is_empty() {
				return Some(ast);
			}
			panic!(
				"attempt to resolve path {} against AST {}",
				path.to_global_path(),
				ast
			)
		}
		for (import, _kind) in value.alias_for.iter() {
			println!("      {}import {}", indent, import);
			let mut new_path = import.to_parts();
			new_path.extend_from_slice(path);
			if let Some(result) = self.lookup_internal(&self.decls, &new_path, depth + 1, checked) {
				return Some(result);
			}
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
			if let Some(result) =
				self.lookup_internal(&self.decls, &new_path, depth + 1, checked)
			{
				return Some(result);
			}
		}
		None
    }

    pub fn print_to(&self, f: &mut dyn fmt::Write) -> fmt::Result {
        writeln!(f, "decls:")?;
        self.decls.for_each(&mut |ident, _decl| {
            writeln!(f, "  - {}", GlobalIdent::from_ident_path(ident)).unwrap();
        });
        writeln!(f, "use_wildcards:")?;
        for wildcard in self.wildcard_imports_temp.iter() {
            writeln!(f, "  - {:?}: {:?}", wildcard.target, wildcard.source)?;
        }
        Ok(())
    }

    pub fn add_type_stub(&mut self, name: &str) {
        let path = parse_str::<Path>(name).unwrap();
        let ident = path.segments.last().unwrap().ident.clone();
        let ast = quote! { pub struct #ident {} };
        let ast = parse2::<Item>(ast.clone()).unwrap_or_else(|_| panic!("{}", ast));

        println!("add type stub {}", name);

        let qualified = &GlobalIdent::from_qualified_name(name);

        self.decls
            .find_or_create(&qualified.parent())
            .add_child(
                qualified.last_part(),
                Binding::new_type_ast(qualified.clone(), ast),
            );
    }
}

#[derive(Debug)]
pub struct WildcardImport {
    pub target: GlobalIdent,
    pub source: GlobalIdent,
}

/// Rust allow sharing one local name sometimes:
/// - mod with functions and consts
/// - derive with traits
/// - types, traits and structs with functions and constants
/// - imports inherit sharing properties of the item it import
#[derive(Debug)]
pub struct Binding {
    pub address: GlobalIdent,
    pub non_type_ast: Option<DeclAst>,
	pub type_ast: Option<DeclAst>,
    /// if this binding is targeted by `use` operator
    pub alias_for: Vec<(GlobalIdent, ImportKind)>,
    /// means this binding imports all children from all these binding
    pub wildcard_alias_for: BTreeSet<GlobalIdent>,
}

impl FromPath<IdentPart> for Binding {
	fn from_path(path: &[IdentPart]) -> Self {
		Self::new_empty(path.to_global_path())
	}
}

impl Binding {
	pub fn new_empty(path: GlobalIdent) -> Self {
		Binding {
			address: path,
			non_type_ast: Default::default(),
			type_ast: Default::default(),
			alias_for: Default::default(),
			wildcard_alias_for: Default::default(),
		}
	}

	pub fn new_type_ast(path: GlobalIdent, item: Item) -> Self {
		Binding {
			address: path.clone(),
			non_type_ast: Default::default(),
			type_ast: Some(DeclAst {
				address: path,
				ast: item,
			}),
			alias_for: Default::default(),
			wildcard_alias_for: Default::default(),
		}
	}

	pub fn new_non_type_ast(path: GlobalIdent, item: Item) -> Self {
		Binding {
			address: path.clone(),
			non_type_ast: Some(DeclAst {
				address: path,
				ast: item,
			}),
			type_ast: Default::default(),
			alias_for: Default::default(),
			wildcard_alias_for: Default::default(),
		}
	}
}

impl Display for Binding {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "({})", self.address)?;
		if let Some(ast) = &self.type_ast {
			write!(f, " TypeAst({})", ast)?;
		}
		write!(f, "({})", self.address)?;
		if let Some(ast) = &self.non_type_ast {
			write!(f, " NonTypeAst({})", ast)?;
		}
		for (source, _) in self.alias_for.iter() {
			write!(f, " Alias({})", source)?;
		}
		for source in self.wildcard_alias_for.iter() {
			write!(f, " Wildcard({})", source)?;
		}
		Ok(())
	}
}

#[derive(Debug)]
pub enum Decl {
    None,
    Ast(DeclAst),
    Import(GlobalIdent, ImportKind),
    Mod(Mod),
}

impl Display for Decl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Decl::None => write!(f, "None"),
            Decl::Ast(it) => write!(f, "Ast({})", it),
            Decl::Import(it, kind) => write!(f, "Import({}, {:?})", it, kind),
            Decl::Mod(it) => write!(f, "Mod({:?})", it),
        }
    }
}

#[derive(Debug)]
pub enum ImportKind {
    Normal,
    Wildcard,
}

impl Default for Decl {
    fn default() -> Self {
        Decl::None
    }
}

#[derive(Debug)]
pub struct Mod {
    pub address: GlobalIdent,
    pub wildcard_imported_mods: BTreeSet<GlobalIdent>,
}

#[derive(Debug)]
pub struct DeclAst {
    pub address: GlobalIdent,
    pub ast: Item,
}

impl Display for DeclAst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}) {}",
            self.address,
            self.ast.dedoc().to_token_stream()
        )
    }
}
