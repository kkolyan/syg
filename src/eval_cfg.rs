use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, visit_mut::VisitMut, Attribute, Item};
use to_vec::ToVec;


pub struct DeleteByCfg;

fn should_be_ignored(attrs: &[Attribute]) -> bool {
	attrs.iter().any(|it| it.to_token_stream().to_string() == quote! {#[cfg(target_arch = "wasm32")]}.to_string())
}

fn visit_items(items: &mut Vec<Item>) {
	for i in (0..items.len()).rev() {
		let ignore = match items.get(i).unwrap() {
			Item::Trait(it) => should_be_ignored(&it.attrs),
			Item::Type(it) => should_be_ignored(&it.attrs),
			_ => false,
		};
		if ignore {
			items.remove(i);
		}
	}
}

impl VisitMut for DeleteByCfg {
	fn visit_item_mod_mut(&mut self, i: &mut syn::ItemMod) {
		if let Some((_, items)) = &mut i.content {
			visit_items(items);
		}
	}
	fn visit_file_mut(&mut self, i: &mut syn::File) {
		visit_items(&mut i.items);
	}
}