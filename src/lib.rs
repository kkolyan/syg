pub mod check_path_resolved;
#[allow(clippy::collapsible_match)]
pub mod inline_types;
pub mod lookup_decl;
pub mod display_utils;
pub mod eval_cfg;
pub mod named_tree;
pub mod dedoc;
pub mod bake_wildcards;
pub mod global_ident;
pub mod ident_part;
pub mod resolve_idents;
use std::fmt::Display;

pub mod add_crate;
pub mod model;
pub mod stopwatch;

pub use global_ident::GlobalIdent;
pub use ident_part::IdentPart;
pub use model::*;

#[extend::ext]
pub impl &str {
    fn add_file_segment(self, segment: impl Display) -> String {
        format!("{}/{}", self, segment)
    }

    fn concat(self, s: impl Display) -> String {
        format!("{}{}", self, s)
    }

    fn add_rust_segment(self, segment: impl Display) -> String {
        format!("{}::{}", self, segment)
    }
}
