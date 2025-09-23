use crate::models::{Country, Product};
use crate::repository::Repository;
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;

const MIN_TOKEN_LENGTH: usize = 3;

pub struct CategorizedProductMatches {
    pub by_category: Vec<HashMap<usize, bool>>,
    pub has_any_matches: bool
}

struct SearchIndex {
    repository: Repository,
    product_tokens: Vec<Vec<String>>,
}

#[derive(Clone)]
pub struct SearchEngine {
    index: Rc<SearchIndex>
}

impl SearchEngine {

    fn normalize_text(text: &str) -> String {
        let mut normalized = String::new();

        for character in text.to_lowercase().chars() {
            if character.is_alphanumeric() || character.is_whitespace() {
                normalized.push(character);
            }
        }

        normalized.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn tokenize(text: &str) -> Vec<String> {
        let normalized = Self::normalize_text(text);
        let mut tokens = Vec::new();

        for word in normalized.split_whitespace() {
            if word.len() >= MIN_TOKEN_LENGTH {
                tokens.push(word.to_string());
            }
        }

        tokens
    }

    fn build_product_search_text(product: &Product, repository: Repository) -> String {
        let mut parts = Vec::new();
        parts.push(product.name);
        parts.push(product.description);

        if let Some(country) = &product.country {
            let country_name = country.display_name();
            parts.push(country_name);
        }

        for &category_index in product.categories {
            if let Some(category) = repository.categories().get(category_index) {
                parts.push(category.name);
                parts.push(category.description);
            }
        }

        parts.join(" ")
    }

    fn build_product_tokens(repository: Repository) -> Vec<Vec<String>> {
        let mut product_tokens = Vec::new();

        for product in repository.products() {
            let search_text = Self::build_product_search_text(product, repository);
            let tokens = Self::tokenize(&search_text);
            product_tokens.push(tokens);
        }

        product_tokens
    }

    fn product_matches_query(&self, product_index: usize, query_tokens: &[String]) -> bool {
        if query_tokens.is_empty() {
            return true;
        }

        let product_tokens = &self.index.product_tokens[product_index];

        'outer: for query_token in query_tokens {
            for product_token in product_tokens {
                if product_token.contains(query_token) || query_token.contains(product_token) {
                    continue 'outer;
                }
            }
            return false;
        }

        true
    }

    fn find_matching_products(&self, query: &str) -> BTreeSet<usize> {
        let query_tokens = Self::tokenize(query);
        let mut matching_products = BTreeSet::new();

        for product_index in 0..self.index.repository.products().len() {
            if self.product_matches_query(product_index, &query_tokens) {
                matching_products.insert(product_index);
            }
        }

        matching_products
    }

    fn categorize_products(&self, matched_products: &BTreeSet<usize>, country_filter: Option<Country>) -> CategorizedProductMatches {
        let categories = self.index.repository.categories();
        let products = self.index.repository.products();
        let mut by_category = vec![HashMap::new(); categories.len()];
        let mut has_any_matches = false;

        for (product_index, product) in products.iter().enumerate() {
            let matches_search = matched_products.contains(&product_index);
            let matches_country = country_filter.is_none() || country_filter == product.country;
            let should_include = matches_search && matches_country;

            for &category_index in product.categories {
                if let Some(category_map) = by_category.get_mut(category_index) {
                    category_map.insert(product_index, should_include);
                    has_any_matches = has_any_matches || should_include;
                }
            }
        }

        CategorizedProductMatches {
            by_category,
            has_any_matches
        }
    }

    pub fn new(repository: Repository) -> Self {
        let product_tokens = Self::build_product_tokens(repository);
        let index = Rc::new(SearchIndex {
            repository,
            product_tokens,
        });

        Self { index }
    }

    pub fn find_by_category(&self, query: &str, country_filter: Option<Country>) -> CategorizedProductMatches {
        let matched_products = if query.trim().is_empty() {
            let mut all_products = BTreeSet::new();
            for index in 0..self.index.repository.products().len() {
                all_products.insert(index);
            }
            all_products
        } else {
            self.find_matching_products(query)
        };

        self.categorize_products(&matched_products, country_filter)
    }
}