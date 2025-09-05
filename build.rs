use std::path::{Path, PathBuf};
use anyhow::{bail, Context, Result};
use phf_codegen::Map;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::LazyLock;
use scraper::{Html, Selector};

include!("src/models/mod.rs");

const BASE_URL: &str = "https://european-alternatives.eu";
const RESOURCES_FILE_NAME: &str = "compiled.gresources";
const UI_XML: &str = include_str!("resources/ui.xml");
const MANIFEST_TOML: &str = include_str!("Cargo.toml");
const RESOURCES_XML: &str = include_str!("resources/resources.gresource.xml.in");

// ===== TRAITS =====

trait StringExtensions {
    fn replace_exactly(&self, from: &str, to: &str, count: usize) -> Result<String>;
}

impl<T> StringExtensions for T
where
    T: AsRef<str>,
{
    fn replace_exactly(&self, from: &str, to: &str, count: usize) -> Result<String> {
        let text = self.as_ref();
        let parts: Vec<&str> = text.split(from).collect();
        let actual_count = parts.len() - 1;

        if actual_count != count {
            bail!(
                "Expected to replace exactly {count} occurrence(s) of '{from}' with '{to}' in '{text}', but found {actual_count}."
            );
        }

        Ok(parts.join(to))
    }
}

// ===== BUILD CONFIGURATION =====

struct BuildConfiguration {
    output_dir: PathBuf,
    icons_dir: PathBuf,
    resources_dir: PathBuf,
    resources_ui_file: PathBuf,
    catalog_file: PathBuf,
    icons_xml_file: PathBuf,
    templates_xml_file: PathBuf,
    resources_xml_file: PathBuf,
    compiled_resources_file: PathBuf,
}

impl BuildConfiguration {
    fn new() -> Result<Self> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let resources_dir = root.join("resources");
        let resources_ui_file = resources_dir.join("ui.xml");
        let output_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
        let icons_dir = output_dir.join("icons");
        let catalog_file = output_dir.join("catalog.rs");
        let resources_xml_file = output_dir.join("resources.xml");
        let icons_xml_file = output_dir.join("icons.xml");
        let templates_xml_file = output_dir.join("templates.xml");
        let compiled_resources_file = output_dir.join(RESOURCES_FILE_NAME);

        std::fs::create_dir_all(&icons_dir)?;

        Ok(Self {
            output_dir,
            resources_dir,
            resources_ui_file,
            icons_dir,
            icons_xml_file,
            templates_xml_file,
            catalog_file,
            resources_xml_file,
            compiled_resources_file,
        })
    }
}

// ===== APPLICATION METADATA =====

#[allow(dead_code)]
struct ApplicationMetadata {
    name: &'static str,
    description: &'static str,
    version: &'static str,
    id: String,
    prefix: String,
    title: String,
    authors: Vec<String>,
    categories: Vec<String>,
    keywords: Vec<String>,
}

impl ApplicationMetadata {
    fn extract_from_cargo() -> Result<Self> {
        let name = env!("CARGO_PKG_NAME");
        let description = env!("CARGO_PKG_DESCRIPTION");
        let version = env!("CARGO_PKG_VERSION");
        let authors = env!("CARGO_PKG_AUTHORS")
            .split(':')
            .map(|s| s.to_string())
            .collect();

        let manifest: toml::Value = toml::from_str(MANIFEST_TOML)
            .context("Failed to parse Cargo.toml")?;
        
        let package = manifest.get("package")
            .context("Missing [package] section in Cargo.toml")?;
        
        let metadata = package.get("metadata")
            .context("Missing [package.metadata] section in Cargo.toml")?;
        
        let categories = Self::extract_string_array(package, "categories")?;
        let keywords = Self::extract_string_array(package, "keywords")?;
        let id = Self::extract_string(metadata, "id")?;
        let prefix = Self::extract_string(metadata, "prefix")?;
        let title = Self::extract_string(metadata, "title")?;

        Ok(Self {
            name,
            description,
            version,
            id,
            prefix,
            title,
            authors,
            categories,
            keywords,
        })
    }

    fn extract_string(value: &toml::Value, key: &str) -> Result<String> {
        value.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .context(format!("Key '{key}' is missing or not a string"))
    }

    fn extract_string_array(value: &toml::Value, key: &str) -> Result<Vec<String>> {
        let array = value
            .get(key)
            .context(format!("Missing key '{key}' in Cargo.toml"))?
            .as_array()
            .context(format!("Key '{key}' is not an array"))?;

        array.iter()
            .enumerate()
            .map(|(i, v)| {
                v.as_str()
                    .map(|s| s.to_string())
                    .context(format!("Element at index {i} in key '{key}' is not a string"))
            })
            .collect()
    }
}

// ===== HTTP CLIENT =====

struct HttpClient;

impl HttpClient {
    fn send_request(url: &str) -> Result<minreq::Response> {
        minreq::get(url)
            .with_header("User-Agent", "eu-catalog-builder/1.0")
            .send()
            .map_err(|_| anyhow::anyhow!("Failed to send request to {url}"))
            .and_then(|response| {
                if response.status_code == 200 {
                    Ok(response)
                } else {
                    bail!("HTTP error {status} from {url}", status = response.status_code)
                }
            })
    }

    fn fetch_text(url: &str) -> Result<String> {
        Ok(Self::send_request(url)?.as_str().map(|s| s.to_string())?)
    }

    fn fetch_bytes(url: &str) -> Result<Vec<u8>> {
        Ok(Self::send_request(url)?.as_bytes().to_vec())
    }

    fn fetch_html(url: &str) -> Result<Html> {
        Self::fetch_text(url).map(|text| Html::parse_document(&text))
    }
}

// ===== DOCUMENT SELECTORS =====

struct DocumentSelectors {
    heading: Selector,
    first_paragraph: Selector,
    category_link: Selector,
    category_icon: Selector,
    product_logo: Selector,
    product_link: Selector,
    product_country_flag: Selector,
    product_country: Selector,
    product_website: Selector,
}

static DOCUMENT_SELECTORS: LazyLock<DocumentSelectors> = LazyLock::new(|| {
    DocumentSelectors {
        heading: Selector::parse("h1").unwrap(),
        first_paragraph: Selector::parse(".prose > p:first-child").unwrap(),
        category_link: Selector::parse("a[href*='/category/']").unwrap(),
        category_icon: Selector::parse("img[src*='/categoryLogo/']").unwrap(),
        product_logo: Selector::parse("img[src*='/productLogo/']").unwrap(),
        product_link: Selector::parse("div > a[href*='/product/']").unwrap(),
        product_country_flag: Selector::parse("img[src*='countryFlags']").unwrap(),
        product_country: Selector::parse("img[src*='countryFlags'] + span").unwrap(),
        product_website: Selector::parse(
            r#"a[href^="http"]:not([href*="european-alternatives.eu"]):not(:where(
                [href*="facebook.com"], [href*="fb.com"],
                [href*="twitter.com"], [href*="x.com"],
                [href*="linkedin.com"], [href*="instagram.com"],
                [href*="youtube.com"], [href*="youtu.be"],
                [href*="mastodon"], [href*="github.com"],
                [href*="gitlab.com"], [href*="tiktok.com"],
                [href*="pinterest.com"], [href*="reddit.com"],
                [href*="snapchat.com"], [href*="discord.com"],
                [href*="telegram.org"]
            )) span"#
        ).unwrap(),
    }
});

// ===== CONCURRENT EXECUTOR =====

struct ConcurrentExecutor;

impl ConcurrentExecutor {
    fn execute_and_collect<T, F, R>(items: Vec<T>, worker: F) -> Result<(Vec<R>, Vec<Icon>)>
    where
        T: Send + 'static,
        F: Fn(T) -> Result<(R, Vec<Icon>)> + Send + Sync + Copy + 'static,
        R: Send + 'static,
    {
        let handles: Vec<_> = items
            .into_iter()
            .map(|item| std::thread::spawn(move || worker(item)))
            .collect();

        let mut results = Vec::new();
        let mut all_icons = Vec::new();

        for handle in handles {
            let (result, icons) = handle.join()
                .map_err(|error| anyhow::anyhow!("Thread panicked: {error:?}"))??;
            results.push(result);
            all_icons.extend(icons);
        }

        Ok((results, all_icons))
    }
}

// ===== URL BUILDERS =====

struct UrlBuilder;

impl UrlBuilder {
    fn build_category_url(slug: &str) -> String {
        format!("{BASE_URL}/category/{slug}")
    }

    fn build_categories_index_url() -> String {
        format!("{BASE_URL}/categories")
    }

    fn extract_slug_from_href(href: &str) -> Option<String> {
        href.split('/').last().map(|s| s.to_string())
    }
}

// ===== FILE SYSTEM HELPERS =====

struct FileSystemHelper;

impl FileSystemHelper {
    fn is_source_newer_than_target(source: &Path, target: &Path) -> Result<bool> {
        if !target.exists() {
            return Ok(false);
        }

        let source_time = source.metadata()?.modified()?;
        let target_time = target.metadata()?.modified()?;
        
        Ok(source_time < target_time)
    }

    fn target_exists_and_is_newer(source: &Path, target: &Path) -> Result<bool> {
        Ok(target.exists() && Self::is_source_newer_than_target(source, target)?)
    }
}

// ===== URL DISCOVERY =====

struct UrlDiscovery;

impl UrlDiscovery {
    fn discover_category_urls() -> Result<Vec<(String, String)>> {
        let document = HttpClient::fetch_html(&UrlBuilder::build_categories_index_url())?;
        let hrefs = DocumentExtractor::collect_unique_href_values(&document, &DOCUMENT_SELECTORS.category_link);

        let results = hrefs
            .into_iter()
            .filter_map(|href| {
                UrlBuilder::extract_slug_from_href(&href)
                    .map(|slug| (href, slug))
            })
            .collect();

        Ok(results)
    }

    fn discover_product_urls(categories: &[Category]) -> Result<HashMap<String, Vec<String>>> {
        let mut product_urls = HashMap::<String, Vec<String>>::new();
        
        for category in categories {
            Self::collect_product_urls_for_category(category, &mut product_urls)?;
        }
        
        Ok(product_urls)
    }

    fn collect_product_urls_for_category(
        category: &Category, 
        product_urls: &mut HashMap<String, Vec<String>>
    ) -> Result<()> {
        let category_url = UrlBuilder::build_category_url(&category.slug);
        let document = HttpClient::fetch_html(&category_url)?;

        for element in document.select(&DOCUMENT_SELECTORS.product_link) {
            if let Some(url) = element.value().attr("href") {
                product_urls.entry(url.to_string())
                    .or_default()
                    .push(category.slug.to_string());
            }
        }

        Ok(())
    }
}

// ===== DOCUMENT EXTRACTOR =====

struct DocumentExtractor;

impl DocumentExtractor {
    fn extract_text(document: &Html, selector: &Selector, context: &str) -> Result<String> {
        document.select(selector)
            .next()
            .map(|element| element.text().collect::<String>().trim().to_string())
            .context(format!("{} not found", context))
    }

    fn extract_attribute(document: &Html, selector: &Selector, attribute: &str, context: &str) -> Result<String> {
        document.select(selector)
            .next()
            .and_then(|element| element.value().attr(attribute))
            .map(|value| value.to_string())
            .context(format!("{} not found", context))
    }

    fn extract_optional_attribute(document: &Html, selector: &Selector, attribute: &str) -> Option<String> {
        document.select(selector)
            .next()
            .and_then(|element| element.value().attr(attribute))
            .map(|value| value.to_string())
    }

    fn collect_unique_href_values(document: &Html, selector: &Selector) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut results = Vec::new();

        for anchor in document.select(selector) {
            if let Some(href) = anchor.value().attr("href") {
                if seen.insert(href) {
                    results.push(href.to_string());
                }
            }
        }

        results
    }
}

// ===== CATEGORY EXTRACTOR =====

struct CategoryExtractor;

impl CategoryExtractor {
    fn extract_all_categories() -> Result<(Vec<Category>, Vec<Icon>)> {
        let category_urls = UrlDiscovery::discover_category_urls()?;
        
        ConcurrentExecutor::execute_and_collect(
            category_urls,
            |(url, slug)| Self::extract_single_category(&url, slug).map(|(cat, icon)| (cat, vec![icon]))
        )
    }

    fn extract_single_category(url: &str, slug: String) -> Result<(Category, Icon)> {
        let document = HttpClient::fetch_html(url)?;
        
        let name = DocumentExtractor::extract_text(&document, &DOCUMENT_SELECTORS.heading, "Category name")?;
        let description = DocumentExtractor::extract_text(&document, &DOCUMENT_SELECTORS.first_paragraph, "Category description")?;
        let icon = Self::extract_category_icon(&document, &name)?;

        let category = Category {
            slug,
            name,
            description,
            icon: icon.name.clone(),
        };

        Ok((category, icon))
    }

    fn extract_category_icon(document: &Html, name: &str) -> Result<Icon> {
        let icon_url = DocumentExtractor::extract_attribute(
            document, 
            &DOCUMENT_SELECTORS.category_icon, 
            "src",
            "Category icon"
        )?;

        Icon::from_url(icon_url, name)
    }
}

// ===== PRODUCT EXTRACTOR =====

struct ProductExtractor;

impl ProductExtractor {
    fn extract_all_products(categories: &[Category]) -> Result<(Vec<Product>, Vec<Icon>)> {
        let product_urls = UrlDiscovery::discover_product_urls(categories)?;
        
        ConcurrentExecutor::execute_and_collect(
            product_urls.into_iter().collect::<Vec<_>>(),
            |(url, categories)| Self::extract_single_product_with_icons(&url, categories)
        )
    }

    fn extract_single_product_with_icons(url: &str, categories: Vec<String>) -> Result<(Product, Vec<Icon>)> {
        let document = HttpClient::fetch_html(url)?;
        let product = Self::extract_product_data(&document, categories)?;
        let icons = Self::extract_product_icons(&document, &product)?;

        Ok((product, icons))
    }

    fn extract_product_data(document: &Html, categories: Vec<String>) -> Result<Product> {
        let name = DocumentExtractor::extract_text(document, &DOCUMENT_SELECTORS.heading, "Product name")?;
        let description = DocumentExtractor::extract_text(document, &DOCUMENT_SELECTORS.first_paragraph, "Product description")?;
        
        let website = Self::extract_product_website(document);
        let country = Self::extract_product_country(document);
        let logo = Self::extract_product_logo_name(document, &name)?;

        Ok(Product {
            logo,
            categories,
            name,
            country,
            website,
            description,
        })
    }

    fn extract_product_website(document: &Html) -> Option<String> {
        document.select(&DOCUMENT_SELECTORS.product_website)
            .next()
            .and_then(|span| span.parent())
            .and_then(|anchor| anchor.value().as_element())
            .and_then(|anchor| anchor.attr("href"))
            .map(|href| href.to_string())
    }

    fn extract_product_country(document: &Html) -> Option<Country> {
        document.select(&DOCUMENT_SELECTORS.product_country)
            .next()
            .and_then(|span| Country::parse(span.text().collect::<String>().trim()))
    }

    fn extract_product_logo_name(document: &Html, product_name: &str) -> Result<Option<String>> {
        Ok(Self::extract_product_logo_icon(document, product_name)?
            .map(|icon| icon.name))
    }

    fn extract_product_icons(document: &Html, product: &Product) -> Result<Vec<Icon>> {
        let mut icons = Vec::new();

        if let Some(logo_icon) = Self::extract_product_logo_icon(document, &product.name)? {
            icons.push(logo_icon);
        }

        if let Some(country) = product.country {
            if let Some(flag_icon) = Self::extract_country_flag_icon(document, country)? {
                icons.push(flag_icon);
            }
        }

        Ok(icons)
    }

    fn extract_product_logo_icon(document: &Html, name: &str) -> Result<Option<Icon>> {
        let logo_url = DocumentExtractor::extract_optional_attribute(
            document, 
            &DOCUMENT_SELECTORS.product_logo, 
            "src"
        );

        match logo_url {
            Some(url) => Ok(Some(Icon::from_url(url, name)?)),
            None => Ok(None),
        }
    }

    fn extract_country_flag_icon(document: &Html, country: Country) -> Result<Option<Icon>> {
        let flag_url = DocumentExtractor::extract_optional_attribute(
            document, 
            &DOCUMENT_SELECTORS.product_country_flag, 
            "src"
        );

        match flag_url {
            Some(url) => Ok(Some(Icon::from_url(url, country.icon())?)),
            None => Ok(None),
        }
    }
}

// ===== CATALOG EXTRACTOR =====

struct CatalogExtractor;

impl CatalogExtractor {
    fn extract_complete_catalog() -> Result<(Vec<Category>, Vec<Product>, Vec<Icon>)> {
        let (categories, category_icons) = CategoryExtractor::extract_all_categories()?;
        let (products, product_icons) = ProductExtractor::extract_all_products(&categories)?;
        
        let all_icons = Self::combine_icon_collections(category_icons, product_icons);
        
        Ok((categories, products, all_icons))
    }

    fn combine_icon_collections(mut category_icons: Vec<Icon>, product_icons: Vec<Icon>) -> Vec<Icon> {
        category_icons.extend(product_icons);
        category_icons
    }
}

// ===== CATALOG CODE GENERATION =====

struct CatalogIndexMaps {
    category_slug_to_index: HashMap<String, usize>,
    product_name_to_index: HashMap<String, usize>,
    products_by_category_index: Vec<Vec<usize>>,
    products_by_country_index: Vec<Vec<usize>>,
}

impl CatalogIndexMaps {
    fn build_from_catalog(categories: &[Category], products: &[Product]) -> Self {
        let category_slug_to_index = Self::build_category_slug_index(categories);
        let product_name_to_index = Self::build_product_name_index(products);
        let products_by_category_index = Self::build_products_by_category_index(
            products,
            &category_slug_to_index,
            categories.len(),
        );
        let products_by_country_index = Self::build_products_by_country_index(products);

        Self {
            category_slug_to_index,
            product_name_to_index,
            products_by_category_index,
            products_by_country_index,
        }
    }

    fn build_category_slug_index(categories: &[Category]) -> HashMap<String, usize> {
        categories
            .iter()
            .enumerate()
            .map(|(index, category)| (category.slug.clone(), index))
            .collect()
    }

    fn build_product_name_index(products: &[Product]) -> HashMap<String, usize> {
        products
            .iter()
            .enumerate()
            .map(|(index, product)| (product.name.clone(), index))
            .collect()
    }

    fn build_products_by_category_index(
        products: &[Product],
        category_slug_to_index: &HashMap<String, usize>,
        categories_count: usize,
    ) -> Vec<Vec<usize>> {
        let mut products_by_category = vec![Vec::new(); categories_count];
        
        for (product_index, product) in products.iter().enumerate() {
            Self::assign_product_to_categories(
                product, 
                product_index, 
                category_slug_to_index, 
                &mut products_by_category
            );
        }
        
        products_by_category
    }

    fn assign_product_to_categories(
        product: &Product,
        product_index: usize,
        category_slug_to_index: &HashMap<String, usize>,
        products_by_category: &mut [Vec<usize>],
    ) {
        for slug in &product.categories {
            if let Some(&category_index) = category_slug_to_index.get(slug) {
                products_by_category[category_index].push(product_index);
            }
        }
    }

    fn build_products_by_country_index(products: &[Product]) -> Vec<Vec<usize>> {
        let mut products_by_country = vec![Vec::new(); Country::COUNT];
        
        for (product_index, product) in products.iter().enumerate() {
            Self::assign_product_to_country(product, product_index, &mut products_by_country);
        }
        
        products_by_country
    }

    fn assign_product_to_country(
        product: &Product,
        product_index: usize,
        products_by_country: &mut [Vec<usize>],
    ) {
        if let Some(country) = product.country {
            products_by_country[country as usize].push(product_index);
        }
    }
}

// ===== CATALOG CODE BUILDER =====

struct CatalogCodeBuilder;

impl CatalogCodeBuilder {
    fn format_indexed_vector_collection<T: Debug>(vectors: &[Vec<T>]) -> String {
        let formatted_vectors = vectors.iter()
            .map(|vector| format!("&{:?}", vector))
            .collect::<Vec<_>>()
            .join(", ");

        format!("&[{formatted_vectors}]")
    }

    fn format_phf_hash_map<K: AsRef<str>>(map: &HashMap<K, usize>) -> String {
        let mut phf_builder = Map::new();
        for (key, value) in map {
            phf_builder.entry(key.as_ref(), value.to_string());
        }
        
        phf_builder.build().to_string()
    }

    fn format_optional_country_field(country: Option<Country>) -> String {
        country
            .map(|country| format!("Some(Country::{country:?})"))
            .unwrap_or_else(|| "None".to_string())
    }

    fn format_category_indices_list(
        product_categories: &[String],
        category_slug_to_index: &HashMap<String, usize>,
    ) -> String {
        product_categories
            .iter()
            .map(|slug| category_slug_to_index.get(slug).unwrap().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_category_struct(category: &Category) -> String {
        format!(
            "Category {{
                slug: {slug:?},
                name: {name:?},
                description: {description:?},
                icon: {icon:?}
            }}",
            slug = category.slug,
            name = category.name,
            description = category.description,
            icon = category.icon
        )
    }

    fn format_product_struct(index_maps: &CatalogIndexMaps, product: &Product) -> String {
        let country = Self::format_optional_country_field(product.country);
        let categories = Self::format_category_indices_list(
            &product.categories, 
            &index_maps.category_slug_to_index
        );

        format!(
            "Product {{
                categories: &[{categories}],
                name: {name:?},
                country: {country},
                website: {website:?},
                description: {description:?},
                logo: {logo:?}
            }}",
            name = product.name,
            website = product.website,
            description = product.description,
            logo = product.logo
        )
    }

    fn format_categories_array(categories: &[Category]) -> String {
        categories.iter()
            .map(Self::format_category_struct)
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_products_array(products: &[Product], index_maps: &CatalogIndexMaps) -> String {
        products.iter()
            .map(|product| Self::format_product_struct(index_maps, product))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn build_catalog_struct_code(
        categories: &[Category],
        products: &[Product],
        index_maps: &CatalogIndexMaps,
    ) -> String {
        let categories_map = Self::format_phf_hash_map(&index_maps.category_slug_to_index);
        let products_map = Self::format_phf_hash_map(&index_maps.product_name_to_index);
        let category_products = Self::format_indexed_vector_collection(&index_maps.products_by_category_index);
        let country_products = Self::format_indexed_vector_collection(&index_maps.products_by_country_index);
        let categories_array = Self::format_categories_array(categories);
        let products_array = Self::format_products_array(products, index_maps);

        format!(
            "Catalog {{
                categories: &[{categories_array}],
                products: &[{products_array}],
                categories_map: {categories_map},
                products_map: {products_map},
                category_products: {category_products},
                country_products: {country_products}
            }}"
        )
    }
}

// ===== CATALOG PROCESSOR =====

struct CatalogProcessor<'a> {
    config: &'a BuildConfiguration,
}

impl<'a> CatalogProcessor<'a> {
    fn new(config: &'a BuildConfiguration) -> Self {
        Self { config }
    }

    fn process_catalog_data(&self) -> Result<Vec<Icon>> {
        if self.should_use_cached_catalog()? {
            Ok(Vec::new())
        } else {
            self.regenerate_catalog_data()
        }
    }

    fn should_use_cached_catalog(&self) -> Result<bool> {
        Ok(self.config.catalog_file.exists())
    }

    fn regenerate_catalog_data(&self) -> Result<Vec<Icon>> {
        let (categories, products, icons) = CatalogExtractor::extract_complete_catalog()?;
        
        self.write_catalog_code_to_file(&categories, &products)?;
        
        Ok(icons)
    }

    fn write_catalog_code_to_file(&self, categories: &[Category], products: &[Product]) -> Result<()> {
        let index_maps = CatalogIndexMaps::build_from_catalog(categories, products);
        let catalog_code = CatalogCodeBuilder::build_catalog_struct_code(categories, products, &index_maps);

        std::fs::write(&self.config.catalog_file, catalog_code)
            .context("Failed to write catalog file")
    }
}

// ===== ICON =====

#[derive(Clone)]
struct Icon {
    url: String,
    filename: String,
    name: String,
    is_svg_format: bool,
}

impl Icon {
    fn from_url(url: String, name: &str) -> Result<Self> {
        let name = heck::AsSnakeCase(name).to_string();
        let filename = format!("{name}.svg");
        let extension = url
            .rsplit('.')
            .next()
            .context("Invalid icon URL")?
            .to_lowercase();

        let is_svg_format = extension == "svg";

        Ok(Self {
            url,
            name,
            filename,
            is_svg_format,
        })
    }

    fn download_bytes(&self) -> Result<Vec<u8>> {
        HttpClient::fetch_bytes(&self.url)
    }

    fn save_to_path(&self, path: &Path) -> Result<()> {
        let bytes = self.download_bytes()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

// ===== ICON HARVESTER =====

struct IconHarvester<'a> {
    icon_registry: HashMap<&'a str, &'a Icon>,
}

impl<'a> IconHarvester<'a> {
    fn new() -> Self {
        Self { 
            icon_registry: HashMap::new(),
        }
    }

    fn register_icons(&mut self, icons: &'a [Icon]) -> &mut Self {
        for icon in icons {
            self.icon_registry.insert(&icon.url, icon);
        }
        self
    }

    fn download_all_to_directory(&self, output_directory: &Path) -> Result<&Self> {
        let download_handles: Vec<_> = self.icon_registry
            .values()
            .map(|&icon| {
                let icon = icon.clone();
                let path = output_directory.join(&icon.filename);
                std::thread::spawn(move || Self::process_single_icon_download(icon, &path))
            })
            .collect();

        for handle in download_handles {
            handle.join().unwrap()?;
        }

        Ok(self)
    }

    fn process_single_icon_download(icon: Icon, path: &Path) -> Result<()> {
        if icon.is_svg_format {
            return icon.save_to_path(path);
        }

        let bytes = icon.download_bytes()?;
        let image = image::load_from_memory(&bytes)?;
        let rgba = image.to_rgba8();
        let vtracer_config = vtracer::Config::default();
        let color_image = vtracer::ColorImage {
            pixels: rgba.as_raw().to_vec(),
            width: rgba.width() as usize,
            height: rgba.height() as usize,
        };
        let svg_content = vtracer::convert(color_image, vtracer_config)
            .map_err(|error| anyhow::anyhow!("vtracer error: {error}"))?;
        
        std::fs::write(path, svg_content.to_string())?;
        Ok(())
    }

    fn build_resources_xml(&self) -> String {
        self.icon_registry
            .values()
            .map(|icon| {
                format!(
                    "<file compressed=\"true\" preprocess=\"xml-stripblanks\" alias=\"{filename}\">{filename}</file>",
                    filename = icon.filename
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn save_resources_xml_to_file(&self, path: &Path) -> Result<String> {
        let xml_content = self.build_resources_xml();
        std::fs::write(path, &xml_content)?;
        Ok(xml_content)
    }
}

// ===== ICON PROCESSOR =====

struct IconProcessor<'a> {
    config: &'a BuildConfiguration,
}

impl<'a> IconProcessor<'a> {
    fn new(config: &'a BuildConfiguration) -> Self {
        Self { config }
    }

    fn process_icons(&self, icons: Vec<Icon>, catalog_data_regenerated: bool) -> Result<(String, bool)> {
        if !catalog_data_regenerated && self.should_use_cached_icons() {
            self.load_cached_icons_xml()
        } else {
            self.regenerate_icons_resources(icons)
        }
    }

    fn should_use_cached_icons(&self) -> bool {
        self.config.icons_xml_file.exists()
    }

    fn load_cached_icons_xml(&self) -> Result<(String, bool)> {
        let xml_content = std::fs::read_to_string(&self.config.icons_xml_file)?;
        Ok((xml_content, false))
    }

    fn regenerate_icons_resources(&self, icons: Vec<Icon>) -> Result<(String, bool)> {
        if icons.is_empty() {
            return Ok((String::new(), false));
        }

        let xml_content = IconHarvester::new()
            .register_icons(&icons)
            .download_all_to_directory(&self.config.icons_dir)?
            .save_resources_xml_to_file(&self.config.icons_xml_file)?;

        Ok((xml_content, true))
    }
}

// ===== TEMPLATE EXTRACTOR =====

struct TemplateExtractor {
    template_regex: Regex,
    extracted_templates: HashMap<String, String>,
}

impl TemplateExtractor {
    fn new() -> Result<Self> {
        let template_regex = Regex::new(r#"(?s)<template\s+class="([^"]+)"[^>]*>.*?</template>"#)?;
        let extracted_templates = HashMap::new();
        Ok(Self { template_regex, extracted_templates })
    }

    fn extract_all_templates(&mut self) -> &mut Self {
        for capture in self.template_regex.captures_iter(UI_XML) {
            if let Some(class_match) = capture.get(1) {
                let class_name = heck::AsSnakeCase(class_match.as_str()).to_string();
                let full_template = capture.get(0).map(|m| m.as_str().to_string()).unwrap_or_default();
                self.extracted_templates.insert(class_name, full_template);
            }
        }
        self
    }

    fn save_template_files_to_directory(&self, output_path: &Path) -> Result<&Self> {
        for (class_name, template_content) in &self.extracted_templates {
            let filename = format!("{class_name}.ui");
            let file_path = output_path.join(filename);
            let formatted_template = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<interface>{template_content}</interface>");
            std::fs::write(file_path, formatted_template)?;
        }
        Ok(self)
    }

    fn build_templates_resources_xml(&self) -> String {
        self.extracted_templates
            .keys()
            .map(|class_name| format!("<file compressed=\"true\" alias=\"{class_name}.ui\">{class_name}.ui</file>"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn save_templates_resources_xml_to_file(&self, path: &Path) -> Result<String> {
        let xml_content = self.build_templates_resources_xml();
        std::fs::write(path, &xml_content)?;
        Ok(xml_content)
    }
}

// ===== TEMPLATE PROCESSOR =====

struct TemplateProcessor<'a> {
    config: &'a BuildConfiguration,
}

impl<'a> TemplateProcessor<'a> {
    fn new(config: &'a BuildConfiguration) -> Self {
        Self { config }
    }

    fn process_templates(&self) -> Result<(String, bool)> {
        if self.should_use_cached_templates()? {
            self.load_cached_templates_xml()
        } else {
            self.regenerate_template_resources()
        }
    }

    fn should_use_cached_templates(&self) -> Result<bool> {
        FileSystemHelper::target_exists_and_is_newer(
            &self.config.resources_ui_file,
            &self.config.templates_xml_file
        )
    }

    fn load_cached_templates_xml(&self) -> Result<(String, bool)> {
        let xml_content = std::fs::read_to_string(&self.config.templates_xml_file)?;
        Ok((xml_content, false))
    }

    fn regenerate_template_resources(&self) -> Result<(String, bool)> {
        let xml_content = TemplateExtractor::new()?
            .extract_all_templates()
            .save_template_files_to_directory(&self.config.output_dir)?
            .save_templates_resources_xml_to_file(&self.config.templates_xml_file)?;

        Ok((xml_content, true))
    }
}

// ===== RESOURCE COMPILER =====

struct ResourceCompiler<'a> {
    config: &'a BuildConfiguration,
    app_prefix: &'a str,
    source_directories: Vec<&'a Path>,
    template_replacements: HashMap<&'a str, &'a str>,
}

impl<'a> ResourceCompiler<'a> {
    fn new(config: &'a BuildConfiguration, app_prefix: &'a str) -> Self {
        Self {
            config,
            app_prefix,
            source_directories: Vec::new(),
            template_replacements: HashMap::new(),
        }
    }

    fn add_template_replacement(mut self, placeholder: &'a str, value: &'a str) -> Self {
        self.template_replacements.insert(placeholder, value);
        self
    }

    fn add_source_directory(mut self, directory: &'a Path) -> Self {
        self.source_directories.push(directory);
        self
    }

    fn compile_resources(self) -> Result<()> {
        let final_xml = self.build_final_resources_xml()?;
        std::fs::write(&self.config.resources_xml_file, &final_xml)?;

        glib_build_tools::compile_resources(
            &self.source_directories,
            self.config.resources_xml_file.to_str().context("Invalid XML path")?,
            self.config.compiled_resources_file.to_str().context("Invalid compiled file path")?,
        );

        Ok(())
    }

    fn build_final_resources_xml(&self) -> Result<String> {
        let mut final_xml = String::from(RESOURCES_XML);
        for (placeholder, value) in &self.template_replacements {
            final_xml = final_xml.replace_exactly(&format!("@{placeholder}@"), value, 1)?;
        }
        final_xml.replace_exactly("@APP_PREFIX@", self.app_prefix, 2)
    }
}

struct CargoEnvironmentVariables;

impl CargoEnvironmentVariables {
    fn emit_build_configuration_flags() {
        println!("cargo:rustc-cfg=runtime");
    }

    fn emit_application_metadata(metadata: &ApplicationMetadata, resources_path: &Path) {
        println!("cargo:rustc-env=APP_NAME={}", metadata.name);
        println!("cargo:rustc-env=APP_DESCRIPTION={}", metadata.description);
        println!("cargo:rustc-env=APP_VERSION={}", metadata.version);
        println!("cargo:rustc-env=APP_ID={}", metadata.id);
        println!("cargo:rustc-env=APP_PREFIX={}", metadata.prefix);
        println!("cargo:rustc-env=APP_TITLE={}", metadata.title);
        println!("cargo:rustc-env=APP_AUTHORS={}", metadata.authors.join(","));
        println!("cargo:rustc-env=APP_RESOURCES={}", resources_path.display());
    }

    fn emit_all_environment_variables(metadata: &ApplicationMetadata, resources_path: &Path) {
        Self::emit_build_configuration_flags();
        Self::emit_application_metadata(metadata, resources_path);
    }
}

// ===== BUILD STATE =====

struct BuildState {
    icons_regenerated: bool,
    templates_regenerated: bool,
    icons_xml_content: String,
    templates_xml_content: String,
}

impl BuildState {
    fn new() -> Self {
        Self {
            icons_regenerated: false,
            templates_regenerated: false,
            icons_xml_content: String::new(),
            templates_xml_content: String::new(),
        }
    }

    fn requires_resource_compilation(&self) -> bool {
        self.icons_regenerated || self.templates_regenerated
    }

    fn update_icons_state(&mut self, xml_content: String, regenerated: bool) {
        self.icons_xml_content = xml_content;
        self.icons_regenerated = regenerated;
    }

    fn update_templates_state(&mut self, xml_content: String, regenerated: bool) {
        self.templates_xml_content = xml_content;
        self.templates_regenerated = regenerated;
    }
}

// ===== BUILD ENVIRONMENT =====

struct BuildEnvironment;

impl BuildEnvironment {
    fn setup_cargo_configuration() -> Result<()> {
        println!("cargo:rustc-check-cfg=cfg(runtime)");
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=resources");
        Ok(())
    }
}

// ===== BUILD PIPELINE =====

struct BuildPipeline {
    config: BuildConfiguration,
    app_metadata: ApplicationMetadata,
}

impl BuildPipeline {
    fn new() -> Result<Self> {
        Ok(Self {
            config: BuildConfiguration::new()?,
            app_metadata: ApplicationMetadata::extract_from_cargo()?,
        })
    }

    fn execute_complete_build(&self) -> Result<()> {
        BuildEnvironment::setup_cargo_configuration()?;

        let mut build_state = BuildState::new();
        
        let icons = self.process_catalog_data()?;
        self.process_icon_resources(&mut build_state, icons)?;
        self.process_template_resources(&mut build_state)?;
        self.compile_final_resources(&build_state)?;
        self.emit_cargo_environment_variables();

        Ok(())
    }

    fn process_catalog_data(&self) -> Result<Vec<Icon>> {
        let processor = CatalogProcessor::new(&self.config);
        processor.process_catalog_data()
    }

    fn process_icon_resources(&self, build_state: &mut BuildState, icons: Vec<Icon>) -> Result<()> {
        let processor = IconProcessor::new(&self.config);
        let catalog_data_regenerated = !icons.is_empty();
        let (xml_content, regenerated) = processor.process_icons(icons, catalog_data_regenerated)?;
        build_state.update_icons_state(xml_content, regenerated);
        Ok(())
    }

    fn process_template_resources(&self, build_state: &mut BuildState) -> Result<()> {
        let processor = TemplateProcessor::new(&self.config);
        let (xml_content, regenerated) = processor.process_templates()?;
        build_state.update_templates_state(xml_content, regenerated);
        Ok(())
    }

    fn compile_final_resources(&self, build_state: &BuildState) -> Result<()> {
        if !build_state.requires_resource_compilation() {
            return Ok(());
        }

        ResourceCompiler::new(&self.config, &self.app_metadata.prefix)
            .add_template_replacement("APP_ICONS", &build_state.icons_xml_content)
            .add_template_replacement("APP_TEMPLATES", &build_state.templates_xml_content)
            .add_source_directory(&self.config.output_dir)
            .add_source_directory(&self.config.resources_dir)
            .add_source_directory(&self.config.icons_dir)
            .compile_resources()
    }

    fn emit_cargo_environment_variables(&self) {
        CargoEnvironmentVariables::emit_all_environment_variables(&self.app_metadata, &self.config.compiled_resources_file);
    }
}

// ===== MAIN =====

fn main() -> Result<()> {
    BuildPipeline::new()?.execute_complete_build()
}
