use super::category::Category;
use super::product::Product;

#[derive(Debug)]
pub struct Catalog {
    pub categories: &'static [Category],
    pub products: &'static [Product],
    pub categories_map: phf::Map<&'static str, usize>,
    pub products_map: phf::Map<&'static str, usize>,
    pub category_products: &'static [&'static [usize]],
    pub country_products: &'static [&'static [usize]],
}