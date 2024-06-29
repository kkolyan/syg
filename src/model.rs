use std::{collections::BTreeMap, fmt::Display, ops::Add};

use crate::list_symbols::{Decl, UseWildcard};

#[derive(Default, Debug)]
pub struct Database {
    pub decls: BTreeMap<GlobalIdent, Decl>,
    pub use_aliases: BTreeMap<GlobalIdent, GlobalIdent>,
    pub use_wildcards: Vec<UseWildcard>,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct GlobalIdent(String);

impl Display for GlobalIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl GlobalIdent {
    pub fn from_path_and_name(path: &[String], name: &str) -> Self {
        let mut s = String::new();
        for it in path {
            s += it;
            s += "::"
        }
        s += name;
        Self(s)
    }
}
