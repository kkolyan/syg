use std::fmt::Display;

pub mod list_symbols;
pub mod model;
pub mod stopwatch;

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
