use crate::constants::APP_CATALOG;

use super::models::{Catalog, Category, Product, Country};

#[derive(Clone, Copy, Debug)]
pub struct Repository {
    catalog: &'static Catalog,
}

impl Repository {
    pub const fn new(catalog: &'static Catalog) -> Self {
        Self { catalog }
    }

    pub const fn countries() -> &'static [Country] {
        Country::all()
    }

    pub fn countries_sorted() -> Vec<Country> {
        let mut countries = Country::all().to_vec();
        countries.sort_by(|country_a, country_b| country_a.slug().cmp(country_b.slug()));
        countries
    }

    pub fn categories_sorted(&self) -> Vec<(usize, &Category)> {
        let mut categories = self.catalog.categories.iter().enumerate().collect::<Vec<_>>();
        categories.sort_by(|(_, category_a), (_, category_b)| category_a.slug.cmp(category_b.slug));
        categories
    }

    pub fn category_products_sorted(&self, category: &Category) -> Option<Vec<(usize, &Product)>> {
        if let Some(category_index) = self.catalog.categories_map.get(category.slug).copied()
            && let Some(product_indices) = self.catalog.category_products.get(category_index)
        {
            let mut products = product_indices
                .iter()
                .copied()
                .map(|product_index| (product_index, &self.catalog.products[product_index]))
                .collect::<Vec<_>>();

            products.sort_by(|(_, product_a), (_, product_b)| product_a.name.cmp(product_b.name));
            Some(products)
        } else {
            None
        }
    }

    pub const fn categories(&self) -> &[Category] {
        self.catalog.categories
    }

    pub const fn products(&self) -> &[Product] {
        self.catalog.products
    }

    pub fn category_index_by_slug(self, slug: &str) -> Option<usize> {
        self.catalog.categories_map.get(slug).copied()
    }

    pub fn product_index_by_name(self, name: &str) -> Option<usize> {
        self.catalog.products_map.get(name).copied()
    }

    pub fn product_by_index(&self, index: usize) -> Option<&Product> {
        self.catalog.products.get(index)
    }

    pub fn category_by_index(&self, index: usize) -> Option<&Category> {
        self.catalog.categories.get(index)
    }

    pub fn product_indices_by_category(&self, category: &Category) -> Option<&[usize]> {
        self.catalog.categories_map
            .get(category.slug)
            .and_then(|category_index| self.catalog.category_products.get(*category_index).copied())
    }

    pub fn product_indices_by_country(&self, country: Country) -> Option<&[usize]> {
        self.catalog
            .country_products
            .get(country as usize)
            .copied()
    }
}

impl Default for Repository {
    fn default() -> Self {
        Self::new(&APP_CATALOG)
    }
}