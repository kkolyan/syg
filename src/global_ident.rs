use std::{collections::VecDeque, fmt::Display};

use syn::{spanned::Spanned, Ident, Path, PathArguments, PathSegment};
use to_vec::ToVec;

use crate::IdentPart;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone, Hash)]
pub struct GlobalIdent(String);

impl Display for GlobalIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&GlobalIdent> for Vec<IdentPart> {
	fn from(val: &GlobalIdent) -> Self {
		val.to_parts()
	}
}

impl GlobalIdent {
    pub fn to_parts(&self) -> Vec<IdentPart> {
		if self.0.is_empty() {
			return vec![];
		}
        self.0
            .split("::")
            .map(IdentPart::from_name)
			.collect()
    }

    pub fn qualify_syn_path(&self, path: &mut Path) {
		for part in self.parent().to_parts().iter().rev() {
            path.segments.insert(
                0,
                PathSegment {
                    ident: Ident::new(part.to_string().as_str(), path.span()),
                    arguments: PathArguments::None,
                },
            );
		}
    }

    pub fn parent(&self) -> GlobalIdent {
		match self.0.rsplit_once("::") {
			Some((parent, _name)) => Self(parent.to_string()),
			None => Self::root(),
		}
    }

	pub fn last_part(&self) -> IdentPart {
		IdentPart::from_name(self.0.rsplit("::").next().unwrap())
	}

    pub fn from_mod_and_path(mod_: &GlobalIdent, path: &[String]) -> Self {
		if path.is_empty() {
			return mod_.clone();
		}
		if mod_.0.is_empty() {
			return Self::from_path(path);
		}
        let mut s = mod_.0.clone();
        for it in path {
            s += "::";
            s += it;
        }
        Self(s)
    }

    pub fn from_mod_and_name(mod_: &GlobalIdent, name: &str) -> Self {
		if mod_.0.is_empty() {
			return Self::from_qualified_name(name);
		}
        Self(mod_.0.clone() + "::" + name)
    }

    pub fn from_path_and_name(path: &[String], name: &str) -> Self {
		if path.is_empty() {
			return Self::from_qualified_name(name);
		}
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

    pub fn from_ident_path(path: &[IdentPart]) -> Self {
        Self(path.iter().map(|it| it.to_string()).to_vec().join("::"))
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

    pub fn from_qualified_name(qualified_name: &str) -> GlobalIdent {
        Self(qualified_name.into())
    }
	
	pub fn root() -> GlobalIdent {
		Self(Default::default())
	}
}
