use std::fmt::Display;

use syn::{spanned::Spanned, Ident, Path, PathArguments, PathSegment};
use to_vec::ToVec;



#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct GlobalIdent(String);

impl Display for GlobalIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl GlobalIdent {
    pub fn qualify_syn_path(&self, path: &mut Path) {
        for item in self.parent().0.split("::").to_vec().into_iter().rev() {
            path.segments.insert(
                0,
                PathSegment {
                    ident: Ident::new(item, path.span()),
                    arguments: PathArguments::None,
                },
            );
        }
    }

    pub fn parent(&self) -> GlobalIdent {
        GlobalIdent(self.0.rsplitn(2, "::").last().unwrap().to_owned())
    }

    pub fn from_mod_and_path(mod_: &GlobalIdent, path: &[String]) -> Self {
        let mut s = mod_.0.clone();
        for it in path {
            s += "::";
            s += it;
        }
        Self(s)
    }

    pub fn from_mod_and_name(mod_: &GlobalIdent, name: &str) -> Self {
        Self(mod_.0.clone() + "::" + name)
    }

    pub fn from_path_and_name(path: &[String], name: &str) -> Self {
        let mut s = String::new();
        for it in path {
            s += it;
            s += "::"
        }
        s += name;
        Self(s)
    }

    pub fn from_path(path: &[String]) -> Self {
        Self(path.join("::"))
    }

    pub fn try_replace_base(&self, from: &GlobalIdent, to: &GlobalIdent) -> Option<GlobalIdent> {
        if self.0.starts_with(&from.0) {
            let from = from.0.clone() + "::";
            if self.0.starts_with(&from) {
                let to = to.0.clone() + "::";
                return Some(Self(self.0.replace(&from, &to)));
            }
        }
        None
    }
	
	pub(crate) fn from_qualified_name(qualified_name: &str) -> GlobalIdent {
		Self(qualified_name.into())
	}
	
}