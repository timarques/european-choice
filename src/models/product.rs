use super::country::Country;
use super::{Categories, String};

#[derive(Debug, Clone)]
pub struct Product {
    pub categories: Categories,
    pub logo: Option<String>,
    pub name: String,
    pub description: String,
    pub country: Option<Country>,
    pub website: Option<String>,
}