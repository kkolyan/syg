use std::fmt::Display;

use syn::{spanned::Spanned, Ident, Path, PathArguments, PathSegment};
use to_vec::ToVec;

use crate::GlobalIdent;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct IdentPart(String);

impl Display for IdentPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl IdentPart {
    pub fn from_name(name: &str) -> Self {
        Self(name.into())
    }

	pub fn from_ident(value: &Ident) -> Self {
		Self(value.to_string())
	}
}

#[extend::ext]
pub impl &[IdentPart] {
	fn to_global_path(&self) -> GlobalIdent {
		GlobalIdent::from_ident_path(self)
	}
}