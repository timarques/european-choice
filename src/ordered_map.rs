use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OrderedMap<T> {
    map: HashMap<usize, T>,
    indices: Vec<usize>,
}

impl <T> Default for OrderedMap<T> {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            indices: Vec::new(),
        }
    }
}

impl<T> OrderedMap<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.indices.iter().filter_map(|&index| {
            self.map.get(&index).map(|value| (index, value))
        })
    }

    pub fn insert(&mut self, key: usize, value: T) -> Option<T> {
        if !self.map.contains_key(&key) {
            self.indices.push(key);
        }
        self.map.insert(key, value)
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        self.map.get(&key)
    }

    pub fn get_by_index(&self, index: usize) -> Option<&T> {
        self.indices.get(index).and_then(|&key| {
            self.map.get(&key)
        })
    }

    pub fn remove(&mut self, key: usize) -> Option<T> {
        if let Some(value) = self.map.remove(&key) {
            self.indices.retain(|&index| index != key);
            Some(value)
        } else {
            None
        }
    }


    pub const fn len(&self) -> usize {
        self.indices.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn first(&self) -> Option<(usize, &T)> {
        self.indices.first().and_then(|&key| {
            self.map.get(&key).map(|value| (key, value))
        })
    }

    pub fn last(&self) -> Option<(usize, &T)> {
        self.indices.last().and_then(|&key| {
            self.map.get(&key).map(|value| (key, value))
        })
    }
}