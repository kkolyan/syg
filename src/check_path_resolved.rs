use std::collections::BTreeSet;

use quote::ToTokens;
use syn::{
    visit::{visit_path, Visit},
    Path, PathResolution,
};

use crate::named_tree::NamedNode;

#[derive(Debug, Default)]
pub struct PathResolutionCheck {
    pub failed: NamedNode<usize, Vec<String>>,
    pub not_attempted: NamedNode<usize, Vec<String>>,
    pub aggregated: NamedNode<usize, Vec<(String, PathResolution)>>,
    stack: Vec<usize>,
    next_id: usize,
}
impl PathResolutionCheck {
    pub fn check_path(path: &Path) -> PathResolutionCheck {
        let mut s = Self::default();
        s.visit_path(path);
        s
    }
}

#[extend::ext]
impl usize {
    fn get_and_inc(&mut self) -> usize {
        let v = *self;
        *self += 1;
        v
    }
}

impl Visit<'_> for PathResolutionCheck {
	fn visit_path_resolution(&mut self, i: &'_ syn::PathResolution) {
	}
	
    fn visit_path(&mut self, i: &'_ syn::Path) {
        self.stack.push(self.next_id.get_and_inc());
        match i.resolution {
            PathResolution::NotAttempted => {
                self.not_attempted
                    .find_or_create(self.stack.clone())
                    .get_value_mut()
                    .push(i.to_token_stream().to_string());
            }
            PathResolution::Failed => {
                self.failed
                    .find_or_create(self.stack.clone())
                    .get_value_mut()
                    .push(i.to_token_stream().to_string());
            }
            PathResolution::Resolved(_) => {}
        }
        self.aggregated
            .find_or_create(self.stack.clone())
            .get_value_mut()
            .push((i.to_token_stream().to_string(), i.resolution.clone()));
        visit_path(self, i);
        self.stack.pop();
    }
}
