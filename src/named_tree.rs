use std::{collections::BTreeMap, fmt::Debug};

#[derive(Debug)]
pub struct NamedNode<K, V> {
	path: Vec<K>,
    value: V,
    children: BTreeMap<K, NamedNode<K, V>>,
}

impl<K, V: Default> Default for NamedNode<K, V> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            children: Default::default(),
            path: Default::default(),
        }
    }
}

impl<K: Ord + Clone + Debug, V: Debug> NamedNode<K, V> {
	pub fn children(&self) -> impl Iterator<Item=&NamedNode<K,V>> {
		self.children.values()
	}

	pub fn path(&self) -> &[K] {
		&self.path
	}

    pub fn get_child(&self, key: &K) -> Option<&NamedNode<K, V>> {
        self.children.get(key)
    }

    pub fn get_child_mut(&mut self, key: &K) -> Option<&mut NamedNode<K, V>> {
        self.children.get_mut(key)
    }

    pub fn get_value(&self) -> &V {
        &self.value
    }

    pub fn get_value_mut(&mut self) -> &mut V {
        &mut self.value
    }

    pub fn set_value(&mut self, value: V) {
        self.value = value
    }

    pub fn add_child(&mut self, key: K, value: V) {
		let mut path = self.path.clone();
		path.push(key.clone());
        self.children.insert(
            key,
            NamedNode {
                value,
                children: Default::default(),
                path,
            },
        );
    }

    pub fn find_mut<'a, 'b>(
        &'a mut self,
        key: impl Into<Vec<K>>,
    ) -> Option<&'a mut NamedNode<K, V>>
    where
        K: 'b,
    {
        let mut n = self;
        for part in key.into() {
            n = n.children.get_mut(&part)?;
        }
        Some(n)
    }

    pub fn find_mut_unchecked<'a, 'b>(
        &'a mut self,
        key: impl Into<Vec<K>>,
    ) -> &'a mut NamedNode<K, V>
    where
        K: 'b,
    {
		let me = format!("{:?} ({:?})", &self.path, &self.value);
		let key = key.into();
		if let Some(v) = self.find_mut(key.clone()) {
			return v;
		}
        panic!("failed to find {:?} against {}", key, me);
    }

    pub fn find_or_create<'a, 'b>(
        &'a mut self,
        key: impl Into<Vec<K>>,
		initializer: impl Fn(&[K]) -> V,
    ) -> &'a mut NamedNode<K, V>
    where
        K: 'b,
    {
        let mut n = self;
        for part in key.into() {
			let mut path = n.path.clone();
			path.push(part.clone());
            n = n.children.entry(part.clone()).or_insert_with(|| NamedNode {
                value: initializer(&path),
                children: Default::default(),
                path,
            });
        }
        n
    }

    pub fn find_value<'a, 'b>(&'a self, key: impl Into<Vec<K>>) -> Option<&V>
    where
        K: 'b,
    {
        let mut n = self;
        for part in key.into() {
            n = n.children.get(&part)?;
        }
        Some(&n.value)
    }

    pub fn for_each_mut(&mut self, f: &mut dyn FnMut(&[K], &mut V, &[K])) {
        Self::for_each_mut_internal(self, &[], f);
    }

    fn for_each_mut_internal(
        node: &mut NamedNode<K, V>,
        path: &[K],
        f: &mut dyn FnMut(&[K], &mut V, &[K]),
    ) {
        for (k, v) in node.children.iter_mut() {
            let mut new_path: Vec<K> = Vec::from(path);
            new_path.push(k.clone());
            f(&new_path, &mut v.value, &v.path);
            Self::for_each_mut_internal(v, &new_path, f)
        }
    }

    pub fn for_each(&self, f: &mut dyn FnMut(&[K], &V)) {
        Self::for_each_internal(self, &[], f);
    }

    fn for_each_internal(node: &NamedNode<K, V>, path: &[K], f: &mut dyn FnMut(&[K], &V)) {
        for (k, v) in node.children.iter() {
            let mut new_path: Vec<K> = Vec::from(path);
            new_path.push(k.clone());
            f(&new_path, &v.value);
            Self::for_each_internal(v, &new_path, f)
        }
    }

	pub fn left_join<V2>(&mut self, other: Option<&NamedNode<K,V2>>, f: &mut dyn FnMut(&mut V, Option<&V2>)) {
		f(&mut self.value, other.map(|it| &it.value));
		for (k, v) in self.children.iter_mut() {
			let o = other.and_then(|it| it.children.get(k));
			v.left_join(o, f);
		}
	}
}


// use std::collections::btree_map::Iter as BTreeIter;
// pub struct Iter<'a, K, V> {
// 	stack: Vec<&'a NamedNode<K,V>>,
// 	name_stack: &'a Vec<K>,
// 	iter: BTreeIter<'a, K, V>,
// }

// impl<'a, K: 'a, V> Iterator for Iter<'a, K, V> {
//     type Item = (&'a [K], &'a V);

//     fn next(&mut self) -> Option<Self::Item> {
// 		if let Some((_k, v)) = self.iter.next() {
// 			return Some((&self.name_stack, v))
// 		}
//         todo!()
//     }
// }
