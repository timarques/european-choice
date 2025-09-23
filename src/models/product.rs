use super::country::Country;
use super::{Categories, String, Array};

#[derive(Debug, Clone)]
pub struct Product {
    pub categories: Categories,
    pub logo: String,
    pub name: String,
    pub description: String,
    pub summary: String,
    pub country: Option<Country>,
    pub websites: Array<(String, String)>
}

impl AsRef<Self> for Product {
    fn as_ref(&self) -> &Self {
        self
    }
}