use syn::{visit_mut::{*}, Item};

#[extend::ext]
pub impl Item {
	fn dedoc(&self) -> Item {
		let mut copy = self.clone();
		DedocVisit.visit_item_mut(&mut copy);
		copy
	}
}

struct DedocVisit;

impl VisitMut for DedocVisit {

	fn visit_item_struct_mut(&mut self, i: &mut syn::ItemStruct) {
		i.attrs.clear();
		visit_item_struct_mut(self, i)
	}

	fn visit_item_trait_mut(&mut self, i: &mut syn::ItemTrait) {
		i.attrs.clear();
		visit_item_trait_mut(self, i)
	}

	fn visit_item_fn_mut(&mut self, i: &mut syn::ItemFn) {
		i.attrs.clear();
		visit_item_fn_mut(self, i)
	}

	fn visit_item_enum_mut(&mut self, i: &mut syn::ItemEnum) {
		i.attrs.clear();
		visit_item_enum_mut(self, i)
	}

	fn visit_item_static_mut(&mut self, i: &mut syn::ItemStatic) {
		i.attrs.clear();
		visit_item_static_mut(self, i)
	}

	fn visit_item_const_mut(&mut self, i: &mut syn::ItemConst) {
		i.attrs.clear();
		visit_item_const_mut(self, i)
	}

	fn visit_item_mod_mut(&mut self, i: &mut syn::ItemMod) {
		i.attrs.clear();
		visit_item_mod_mut(self, i)
	}

	fn visit_field_mut(&mut self, i: &mut syn::Field) {
		i.attrs.clear();
		visit_field_mut(self, i)
	}

	fn visit_trait_item_fn_mut(&mut self, i: &mut syn::TraitItemFn) {
		i.attrs.clear();
		visit_trait_item_fn_mut(self, i)
	}

	fn visit_item_type_mut(&mut self, i: &mut syn::ItemType) {
		i.attrs.clear();
		visit_item_type_mut(self, i)
	}
}