use super::models::{Catalog, Category};

pub struct Repository<'a> {
    catalog: &'a Catalog
}

impl <'a> Repository <'a> {

    pub fn new(catalog: &'a Catalog) -> Self {
        Self { catalog }
    }

    pub fn categories(&self) -> &'static [Category] {
        self.catalog.categories
    }

}