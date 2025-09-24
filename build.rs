use anyhow::{Context, Result, bail};
use phf_codegen::Map;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Write};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

include!("src/models/mod.rs");

const BASE_URL: &str = "https://european-alternatives.eu";
const RESOURCES_FILE_NAME: &str = "compiled.gresources";
const UI_XML: &str = include_str!("data/ui.xml");
const MANIFEST_TOML: &str = include_str!("Cargo.toml");
const RESOURCES_XML: &str = include_str!("data/resources.xml.in");

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

struct Paths {
    data_dir: PathBuf,
    ui_file: PathBuf,
    style_file: PathBuf,
    #[allow(dead_code)]
    icon_file: PathBuf,

    output_dir: PathBuf,
    output_icons_dir: PathBuf,
    output_catalog_file: PathBuf,
    output_resources_file: PathBuf,
    output_icons_file: PathBuf,
    output_templates_file: PathBuf,
    output_resources_compiled_file: PathBuf,
}

impl Paths {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let data_dir = root.join("data");
        let ui_file = data_dir.join("ui.xml");
        let style_file = data_dir.join("style.css");
        let icon_file = if cfg!(windows) {
            data_dir.join("icon.ico")
        } else {
            data_dir.join("icon.svg")
        };

        let output_dir = PathBuf::from(
            std::env::var("OUTPUT_DIR").unwrap_or_else(|_| std::env::var("OUT_DIR").unwrap()),
        );
        let output_icons_dir = output_dir.join("icons");
        let output_catalog_file = output_dir.join("catalog.rs");
        let output_resources_file = output_dir.join("resources.xml");
        let output_icons_file = output_dir.join("icons.xml");
        let output_templates_file = output_dir.join("templates.xml");
        let output_resources_compiled_file = output_dir.join(RESOURCES_FILE_NAME);

        Self {
            data_dir,
            ui_file,
            style_file,
            icon_file,

            output_dir,
            output_icons_dir,
            output_catalog_file,
            output_resources_file,
            output_icons_file,
            output_templates_file,
            output_resources_compiled_file,
        }
    }
}

// ===== APPLICATION METADATA =====

#[allow(dead_code)]
struct Metadata {
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

impl Metadata {
    fn extract_from_cargo() -> Result<Self> {
        let name = env!("CARGO_PKG_NAME");
        let description = env!("CARGO_PKG_DESCRIPTION");
        let version = env!("CARGO_PKG_VERSION");
        let authors = env!("CARGO_PKG_AUTHORS")
            .split(':')
            .map(std::string::ToString::to_string)
            .collect();

        let manifest: toml::Value =
            toml::from_str(MANIFEST_TOML).context("Failed to parse Cargo.toml")?;
        let package = manifest
            .get("package")
            .context("Missing [package] section in Cargo.toml")?;
        let metadata = package
            .get("metadata")
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
        value
            .get(key)
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .context(format!("Key '{key}' is missing or not a string"))
    }

    fn extract_string_array(value: &toml::Value, key: &str) -> Result<Vec<String>> {
        let array = value
            .get(key)
            .context(format!("Missing key '{key}' in Cargo.toml"))?
            .as_array()
            .context(format!("Key '{key}' is not an array"))?;

        array
            .iter()
            .enumerate()
            .map(|(i, v)| {
                v.as_str().map(std::string::ToString::to_string).context(format!(
                    "Element at index {i} in key '{key}' is not a string"
                ))
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
                    bail!(
                        "HTTP error {status} from {url}",
                        status = response.status_code
                    )
                }
            })
    }

    fn fetch_text(url: &str) -> Result<String> {
        Ok(Self::send_request(url)?
            .as_str()
            .map(std::string::ToString::to_string)?)
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
    title_tag: Selector,
    category_link: Selector,
    category_icon: Selector,
    product_prose: Selector,
    product_logo: Selector,
    product_link: Selector,
    product_country: Selector,
    product_other_websites: Selector,
    product_website: Selector,
}

static DOCUMENT_SELECTORS: LazyLock<DocumentSelectors> = LazyLock::new(|| DocumentSelectors {
    heading: Selector::parse("h1").unwrap(),
    first_paragraph: Selector::parse(".prose > p:first-child").unwrap(),
    title_tag: Selector::parse("title").unwrap(),
    category_link: Selector::parse("a[href*='/category/']").unwrap(),
    category_icon: Selector::parse("img[src*='/categoryLogo/']").unwrap(),
    product_prose: Selector::parse(".prose").unwrap(),
    product_logo: Selector::parse("img[src*='/productLogo/']").unwrap(),
    product_link: Selector::parse("div > a[href*='/product/']").unwrap(),
    product_country: Selector::parse("img[src*='countryFlags'] + span").unwrap(),
    product_other_websites: Selector::parse("article .items-center a").unwrap(),
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
            )) span"#,
    )
    .unwrap(),
});

// ===== CONCURRENT EXECUTOR =====

struct ConcurrentExecutor;

impl ConcurrentExecutor {
    fn is_single_threaded() -> bool {
        std::env::var("SINGLE_THREAD_BUILD")
            .map(|value| value.to_lowercase() == "true")
            .unwrap_or(false)
    }

    fn execute_and_collect<I, T, F, R>(items: I, worker: F) -> Result<(Vec<R>, Vec<Icon>)>
    where
        I: IntoIterator<Item = T>,
        T: Send + 'static,
        F: Fn(T) -> Result<(R, Vec<Icon>)> + Send + Sync + Copy + 'static,
        R: Send + 'static,
    {
        let items: Vec<T> = items.into_iter().collect();
        let mut results = Vec::with_capacity(items.len());
        let mut all_icons = Vec::with_capacity(items.len());

        if Self::is_single_threaded() {
            for item in items {
                let (result, icons) = worker(item)?;
                results.push(result);
                all_icons.extend(icons);
            }
        } else {
            let handles = items
                .into_iter()
                .map(|item| std::thread::spawn(move || worker(item)))
                .collect::<Vec<_>>();

            for handle in handles {
                let (result, icons) = handle
                    .join()
                    .map_err(|error| anyhow::anyhow!("Thread panicked: {error:?}"))??;
                results.push(result);
                all_icons.extend(icons);
            }
        }

        Ok((results, all_icons))
    }

    fn execute_parallel<I, T, F>(items: I, worker: F) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        T: Send + 'static,
        F: Fn(T) -> Result<()> + Send + Sync + Copy + 'static,
    {
        let items: Vec<T> = items.into_iter().collect();

        if Self::is_single_threaded() {
            for item in items {
                worker(item)?;
            }
        } else {
            let handles = items
                .into_iter()
                .map(|item| std::thread::spawn(move || worker(item)))
                .collect::<Vec<_>>();

            for handle in handles {
                handle
                    .join()
                    .map_err(|error| anyhow::anyhow!("Thread panicked: {error:?}"))??;
            }
        }

        Ok(())
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
        href.split('/')
            .next_back()
            .map(std::string::ToString::to_string)
    }
}

// ===== FILE SYSTEM HELPERS =====

struct FileSystemHelper;

impl FileSystemHelper {
    fn is_source_newer_than_target(source: &Path, target: &Path) -> Result<bool> {
        let source_time = source.metadata()?.modified()?;
        let target_time = target.metadata()?.modified()?;

        Ok(source_time > target_time)
    }

    fn target_exists_and_is_newer(source: &Path, target: &Path) -> Result<bool> {
        Ok(
            source.exists()
                && target.exists()
                && Self::is_source_newer_than_target(source, target)?,
        )
    }
}

// ===== DOCUMENT EXTRACTOR =====

struct DocumentExtractor;

impl DocumentExtractor {
    fn extract_text(document: &Html, selector: &Selector, context: &str) -> Result<String> {
        document
            .select(selector)
            .next()
            .map(|element| element.text().collect::<String>().trim().to_string())
            .context(format!("{context} not found"))
    }

    fn extract_attribute(
        document: &Html,
        selector: &Selector,
        attribute: &str,
        context: &str,
    ) -> Result<String> {
        document
            .select(selector)
            .next()
            .and_then(|element| element.value().attr(attribute))
            .map(std::string::ToString::to_string)
            .context(format!("{context} not found"))
    }

    fn extract_optional_attribute(
        document: &Html,
        selector: &Selector,
        attribute: &str,
    ) -> Option<String> {
        document
            .select(selector)
            .next()
            .and_then(|element| element.value().attr(attribute))
            .map(std::string::ToString::to_string)
    }

    fn collect_unique_href_values(document: &Html, selector: &Selector) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut results = Vec::new();

        for anchor in document.select(selector) {
            if
                let Some(href) = anchor.value().attr("href")
                && seen.insert(href)
            {
                results.push(href.to_string());
            }
        }

        results
    }
}

// ===== CATEGORY EXTRACTOR =====

struct CategoryExtractor;

impl CategoryExtractor {
    fn extract_all_categories() -> Result<(Vec<Category>, Vec<Icon>)> {
        let category_urls = Self::discover_category_urls()?;

        ConcurrentExecutor::execute_and_collect(category_urls, |(url, slug)| {
            Self::extract_single_category(&url, slug).map(|(cat, icon)| (cat, vec![icon]))
        })
    }

    fn discover_category_urls() -> Result<HashMap<String, String>> {
        let document = HttpClient::fetch_html(&UrlBuilder::build_categories_index_url())?;
        let hrefs = DocumentExtractor::collect_unique_href_values(
            &document,
            &DOCUMENT_SELECTORS.category_link,
        );
        let results = hrefs
            .into_iter()
            .filter_map(|href| UrlBuilder::extract_slug_from_href(&href).map(|slug| (href, slug)))
            .collect();

        Ok(results)
    }

    fn remove_european_prefix(name: &str) -> Option<String> {
        if name.len() >= 10
            && name.chars().take(8).collect::<String>().to_lowercase() == "european"
            && name.chars().nth(8) == Some(' ')
        {
            let remaining_chars = name.chars().skip(9).collect::<Vec<char>>();
            if let Some(first_remaining_char) = remaining_chars.first() {
                return Some(format!(
                    "{}{}",
                    first_remaining_char.to_uppercase(),
                    remaining_chars.iter().skip(1).collect::<String>()
                ));
            }
        }
        None
    }

    fn extract_single_category(url: &str, slug: String) -> Result<(Category, Icon)> {
        let document = HttpClient::fetch_html(url)?;
        let name = DocumentExtractor::extract_text(
            &document,
            &DOCUMENT_SELECTORS.heading,
            "Category name",
        )?;
        let name = Self::remove_european_prefix(&name).unwrap_or(name);
        let description = DocumentExtractor::extract_text(
            &document,
            &DOCUMENT_SELECTORS.first_paragraph,
            "Category description",
        )?;
        let summary = description
            .split('.')
            .next()
            .map_or_else(|| description.clone(), |s| format!("{s}."));
        let icon = Self::extract_category_icon(&document, &name)?;
        let category = Category {
            slug,
            name,
            description,
            summary,
            icon: icon.name.clone(),
        };

        Ok((category, icon))
    }

    fn extract_category_icon(document: &Html, name: &str) -> Result<Icon> {
        let icon_url = DocumentExtractor::extract_attribute(
            document,
            &DOCUMENT_SELECTORS.category_icon,
            "src",
            "Category icon",
        )?;

        Icon::from_url(icon_url, name)
    }
}

// ===== PRODUCT EXTRACTOR =====

struct ProductExtractor;

impl ProductExtractor {
    fn extract_all_products(categories: &[Category]) -> Result<(Vec<Product>, Vec<Icon>)> {
        let product_urls = Self::discover_product_urls(categories)?;
        ConcurrentExecutor::execute_and_collect(product_urls, |(url, categories)| {
            Self::extract_single_product_with_icons(&url, categories)
        })
    }

    fn discover_product_urls(categories: &[Category]) -> Result<HashMap<String, HashSet<String>>> {
        let mut product_urls = HashMap::new();

        for category in categories {
            Self::collect_product_urls_for_category(&mut product_urls, category)?;
        }

        Ok(product_urls)
    }

    fn collect_product_urls_for_category(
        product_urls: &mut HashMap<String, HashSet<String>>,
        category: &Category,
    ) -> Result<()> {
        let category_url = UrlBuilder::build_category_url(&category.slug);
        let document = HttpClient::fetch_html(&category_url)?;

        for element in document.select(&DOCUMENT_SELECTORS.product_link) {
            if let Some(url) = element.value().attr("href") {
                let category_list = product_urls.entry(url.to_string()).or_default();

                category_list.insert(category.slug.to_string());
            }
        }

        Ok(())
    }

    fn extract_single_product_with_icons(
        url: &str,
        categories: HashSet<String>,
    ) -> Result<(Product, Vec<Icon>)> {
        let document = HttpClient::fetch_html(url)?;
        let product = Self::extract_product_data(&document, categories, url)?;
        let icons = Self::extract_product_icons(&document, &product)?;

        Ok((product, icons))
    }

    fn extract_product_data(
        document: &Html,
        categories: HashSet<String>,
        url: &str,
    ) -> Result<Product> {
        let name =
            DocumentExtractor::extract_text(document, &DOCUMENT_SELECTORS.heading, "Product name")?;
        let name = heck::AsTitleCase(name).to_string();
        let source_website = url.to_string();
        let (description, summary) = Self::extract_description_and_summary(document)?;
        let country = Self::extract_product_country(document);
        let logo = Self::extract_product_logo_name(document, &name)?;
        let categories = categories.into_iter().collect();
        let websites = Self::extract_websites(document, &source_website);

        Ok(Product {
            categories,
            logo,
            name,
            description,
            summary,
            country,
            websites,
        })
    }

    fn extract_description_and_summary(document: &Html) -> Result<(String, String)> {
        let description_element = document
            .select(&DOCUMENT_SELECTORS.product_prose)
            .next()
            .context("Product description not found")?;

        let mut description = String::new();
        for child in description_element.children() {
            let Some(child_element) = child.value().as_element() else {
                continue;
            };
            let element_ref = ElementRef::wrap(child).expect("Child is an element");
            let text = element_ref.text().collect::<String>();
            let trimmed_text = text.trim();

            match child_element.name() {
                "p" if description.is_empty() => description.push_str(trimmed_text),
                "p" => write!(description, "\n\n{trimmed_text}").unwrap(),
                _ => break,
            }
        }

        let summary = Self::generate_summary(&description);

        Ok((description, summary))
    }

    fn generate_summary(description: &str) -> String {
        let mut summary = String::new();
        let mut sentence_count = 0;

        for sentence in description.replace("\n\n", "\n").split('.') {
            let trimmed_sentence = sentence.trim();
            if !trimmed_sentence.is_empty() && sentence_count < 2 {
                write!(summary, "{trimmed_sentence}.").unwrap();
                sentence_count += 1;

                if sentence_count == 2 {
                    break;
                }
            }
        }

        summary
    }

    fn extract_websites(document: &Html, source: &str) -> Vec<(String, String)> {
        let company_website_option = Self::extract_product_website(document);
        let mut websites = company_website_option.map_or_else(
            || vec![(String::from("European Alternatives"), source.to_string())],
            |oficial_website| {
                vec![
                    (String::from("Company"), oficial_website),
                    (String::from("European Alternatives"), source.to_string()),
                ]
            },
        );

        for element in document.select(&DOCUMENT_SELECTORS.product_other_websites) {
            if let Some(href) = element.value().attr("href")
                && let Some(title) = element.select(&DOCUMENT_SELECTORS.title_tag).next()
            {
                let title = title.text().collect::<String>().trim().to_string();
                websites.push((title, href.to_string().trim().to_string()));
            }
        }

        websites
    }

    fn extract_product_website(document: &Html) -> Option<String> {
        document
            .select(&DOCUMENT_SELECTORS.product_website)
            .next()
            .and_then(|span| span.parent())
            .and_then(|anchor| anchor.value().as_element())
            .and_then(|anchor| anchor.attr("href"))
            .map(std::string::ToString::to_string)
    }

    fn extract_product_country(document: &Html) -> Option<Country> {
        document
            .select(&DOCUMENT_SELECTORS.product_country)
            .next()
            .and_then(|span| Country::parse(span.text().collect::<String>().trim()))
    }

    fn extract_product_logo_name(document: &Html, product_name: &str) -> Result<String> {
        Self::extract_product_logo_icon(document, product_name).map(|icon| icon.name)
    }

    fn extract_product_icons(document: &Html, product: &Product) -> Result<Vec<Icon>> {
        let icon = Self::extract_product_logo_icon(document, &product.name)?;
        Ok(vec![icon])
    }

    fn extract_product_logo_icon(document: &Html, name: &str) -> Result<Icon> {
        let url = DocumentExtractor::extract_optional_attribute(
            document,
            &DOCUMENT_SELECTORS.product_logo,
            "src",
        )
        .context("Product logo not found")?;

        Icon::from_url(url, name)
    }
}

// ===== CATALOG EXTRACTOR =====

struct CatalogExtractor;

impl CatalogExtractor {
    fn extract_complete_catalog() -> Result<(Vec<Category>, Vec<Product>, Vec<Icon>)> {
        let (categories, category_icons) = CategoryExtractor::extract_all_categories()?;
        let (products, product_icons) = ProductExtractor::extract_all_products(&categories)?;
        let country_icons = Self::extract_country_flags_icons()?;

        let icons = category_icons
            .into_iter()
            .chain(product_icons)
            .chain(country_icons)
            .collect::<Vec<_>>();

        Ok((categories, products, icons))
    }

    fn extract_country_flags_icons() -> Result<Vec<Icon>> {
        let mut icons = Vec::with_capacity(Country::COUNT);
        for country in Country::all() {
            let flag_url = format!(
                "https://cdn.european-alternatives.eu/countryFlags/4x3/{code}.svg",
                code = country.code()
            );
            let icon = Icon::from_url(flag_url, country.slug())?;
            icons.push(icon);
        }

        Ok(icons)
    }
}

// ===== CATALOG CODE GENERATION =====

#[allow(clippy::struct_field_names)]
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
                &mut products_by_category,
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
        let formatted_vectors = vectors
            .iter()
            .map(|vector| format!("&{vector:?}"))
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
        country.map_or_else(
            || "None".to_string(),
            |country| format!("Some(crate::models::Country::{country:?})"),
        )
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
            "crate::models::Category {{
                slug: {slug:?},
                name: {name:?},
                summary: {summary:?},
                description: {description:?},
                icon: {icon:?}
            }}",
            slug = category.slug,
            name = category.name,
            summary = category.summary,
            description = category.description,
            icon = category.icon
        )
    }

    fn format_product_struct(index_maps: &CatalogIndexMaps, product: &Product) -> String {
        let country = Self::format_optional_country_field(product.country);
        let categories = Self::format_category_indices_list(
            &product.categories,
            &index_maps.category_slug_to_index,
        );

        format!(
            "crate::models::Product {{
                categories: &[{categories}],
                name: {name:?},
                country: {country},
                description: {description:?},
                summary: {summary:?},
                logo: {logo:?},
                websites: &{websites:?}
            }}",
            name = product.name,
            description = product.description,
            summary = product.summary,
            logo = product.logo,
            websites = product.websites,
        )
    }

    fn format_categories_array(categories: &[Category]) -> String {
        categories
            .iter()
            .map(Self::format_category_struct)
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_products_array(products: &[Product], index_maps: &CatalogIndexMaps) -> String {
        products
            .iter()
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
        let category_products =
            Self::format_indexed_vector_collection(&index_maps.products_by_category_index);
        let country_products =
            Self::format_indexed_vector_collection(&index_maps.products_by_country_index);
        let categories_array = Self::format_categories_array(categories);
        let products_array = Self::format_products_array(products, index_maps);

        format!(
            "crate::models::Catalog {{
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
    paths: &'a Paths,
}

impl<'a> CatalogProcessor<'a> {
    const fn new(paths: &'a Paths) -> Self {
        Self { paths }
    }

    fn process_catalog_data(&self) -> Result<(Vec<Icon>, bool)> {
        if self.should_use_cached_catalog() {
            Ok((vec![], false))
        } else {
            self.regenerate_catalog_data()
        }
    }

    fn should_use_cached_catalog(&self) -> bool {
        self.paths.output_catalog_file.exists()
    }

    fn regenerate_catalog_data(&self) -> Result<(Vec<Icon>, bool)> {
        let (categories, products, icons) = CatalogExtractor::extract_complete_catalog()?;

        self.write_catalog_code_to_file(&categories, &products)?;

        Ok((icons, true))
    }

    fn write_catalog_code_to_file(
        &self,
        categories: &[Category],
        products: &[Product],
    ) -> Result<()> {
        let index_maps = CatalogIndexMaps::build_from_catalog(categories, products);
        let catalog_code =
            CatalogCodeBuilder::build_catalog_struct_code(categories, products, &index_maps);

        std::fs::write(&self.paths.output_catalog_file, catalog_code)
            .context("Failed to write catalog file")
    }
}

// ===== ICON =====

#[derive(Clone)]
struct Icon {
    url: String,
    filename: String,
    name: String,
    extension: String,
}

impl Icon {
    fn from_url(url: String, name: &str) -> Result<Self> {
        let name = heck::AsSnakeCase(name).to_string();
        let extension = url
            .rsplit('.')
            .next()
            .context("Invalid icon URL")?
            .to_lowercase();

        let final_extension = if extension == "svg" { "svg" } else { "png" };

        let filename = format!("{name}.{final_extension}");

        Ok(Self {
            url,
            filename,
            name,
            extension,
        })
    }

    fn is_svg(&self) -> bool {
        self.extension == "svg"
    }

    fn is_png(&self) -> bool {
        self.extension == "png"
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
        let icons_to_download = self
            .icon_registry
            .values()
            .map(|&icon| (icon.clone(), output_directory.to_owned()))
            .collect::<Vec<_>>();

        ConcurrentExecutor::execute_parallel(icons_to_download, |(icon, directory)| {
            Self::download_icon_as_svg(&icon, &directory)
        })?;

        Ok(self)
    }

    fn download_icon_as_svg(icon: &Icon, directory: &Path) -> Result<()> {
        let bytes = HttpClient::fetch_bytes(&icon.url)?;
        let path = directory.join(&icon.filename);
        if icon.is_svg() {
            Self::normalize_svg_bytes(&path, &bytes)
        } else {
            Self::convert_image_to_png(icon, &path, &bytes)
        }?;

        Ok(())
    }

    fn normalize_svg_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
        let options = resvg::usvg::Options::default();
        let write_options = resvg::usvg::WriteOptions::default();
        let tree = resvg::usvg::Tree::from_data(bytes, &options)?;

        let xml = tree.to_string(&write_options);
        Ok(std::fs::write(path, xml)?)
    }

    fn convert_image_to_png(icon: &Icon, path: &Path, bytes: &[u8]) -> Result<()> {
        if icon.is_png() {
            std::fs::write(path, bytes)?;
        } else {
            let image = image::load_from_memory(bytes)?;
            image.save_with_format(path, image::ImageFormat::Png)?;
        }

        Ok(())
    }

    fn build_resources_xml(&self) -> String {
        self.icon_registry
            .values()
            .map(|icon| {
                if icon.is_svg() {
                    format!(
                        "<file compressed=\"true\" preprocess=\"xml-stripblanks\" alias=\"{filename}\">{filename}</file>",
                        filename = icon.filename
                    )
                } else {
                    format!(
                        "<file compressed=\"true\" alias=\"{filename}\">{filename}</file>",
                        filename = icon.filename
                    )
                }
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
    paths: &'a Paths,
}

impl<'a> IconProcessor<'a> {
    const fn new(paths: &'a Paths) -> Self {
        Self { paths }
    }

    fn should_process_icons(&self, icons: &[Icon]) -> bool {
        !icons.is_empty() || !self.paths.output_icons_file.exists()
    }

    fn load_cached_icons_xml(&self) -> Result<(String, bool)> {
        let xml_content = std::fs::read_to_string(&self.paths.output_icons_file)?;
        Ok((xml_content, false))
    }

    fn generate_icons_xml(&self, icons: &[Icon]) -> Result<(String, bool)> {
        std::fs::create_dir_all(&self.paths.output_icons_dir)?;

        let mut harvester = IconHarvester::new();
        let xml_content = harvester
            .register_icons(icons)
            .download_all_to_directory(&self.paths.output_icons_dir)?
            .save_resources_xml_to_file(&self.paths.output_icons_file)?;

        Ok((xml_content, true))
    }

    fn process_icons(&self, icons: &[Icon]) -> Result<(String, bool)> {
        if self.should_process_icons(icons) {
            self.generate_icons_xml(icons)
        } else {
            self.load_cached_icons_xml()
        }
    }
}

// ===== TEMPLATE EXTRACTOR =====

struct TemplateExtractor {
    template_regex: Regex,
    extracted_templates: HashMap<String, String>,
}

impl TemplateExtractor {
    fn new() -> Result<Self> {
        let template_regex =
            Regex::new(r#"(?s)<template\s+class="([^"]+)"[^>]*>.*?</template>"#)
                .map_err(|error| anyhow::anyhow!("Regex compilation error: {error}"))?;
        let extracted_templates = HashMap::new();
        Ok(Self {
            template_regex,
            extracted_templates,
        })
    }

    fn extract_all_templates(&mut self) -> &mut Self {
        for capture in self.template_regex.captures_iter(UI_XML) {
            if let Some(class_match) = capture.get(1) {
                let class_name = heck::AsSnakeCase(class_match.as_str()).to_string();
                let full_template = capture
                    .get(0)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                self.extracted_templates.insert(class_name, full_template);
            }
        }
        self
    }

    fn save_template_files_to_directory(&self, output_path: &Path) -> Result<&Self> {
        for (class_name, template_content) in &self.extracted_templates {
            let filename = format!("{class_name}.ui");
            let file_path = output_path.join(filename);
            let formatted_template = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<interface>{template_content}</interface>"
            );
            std::fs::write(file_path, formatted_template)?;
        }
        Ok(self)
    }

    fn build_templates_resources_xml(&self) -> String {
        self.extracted_templates
            .keys()
            .map(|class_name| {
                format!(
                    "<file compressed=\"false\" alias=\"{class_name}.ui\">{class_name}.ui</file>"
                )
            })
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
    paths: &'a Paths,
}

impl<'a> TemplateProcessor<'a> {
    const fn new(paths: &'a Paths) -> Self {
        Self { paths }
    }

    fn process_templates(&self) -> Result<(String, bool)> {
        if self.should_use_cached_templates()? {
            self.load_cached_templates_xml()
        } else {
            self.regenerate_template_resources()
        }
    }

    fn should_use_cached_templates(&self) -> Result<bool> {
        Ok(self.paths.output_templates_file.exists()
            && FileSystemHelper::target_exists_and_is_newer(
                &self.paths.output_templates_file,
                &self.paths.ui_file,
            )?)
    }

    fn load_cached_templates_xml(&self) -> Result<(String, bool)> {
        let xml_content = std::fs::read_to_string(&self.paths.output_templates_file)?;
        Ok((xml_content, false))
    }

    fn regenerate_template_resources(&self) -> Result<(String, bool)> {
        let xml_content = TemplateExtractor::new()?
            .extract_all_templates()
            .save_template_files_to_directory(&self.paths.output_dir)?
            .save_templates_resources_xml_to_file(&self.paths.output_templates_file)?;

        Ok((xml_content, true))
    }
}

// ===== RESOURCE COMPILER =====

struct ResourceCompiler<'a> {
    paths: &'a Paths,
    app_prefix: &'a str,
    source_directories: Vec<&'a Path>,
    template_replacements: HashMap<&'a str, &'a str>,
}

impl<'a> ResourceCompiler<'a> {
    fn new(paths: &'a Paths, app_prefix: &'a str) -> Self {
        Self {
            paths,
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
        std::fs::write(&self.paths.output_resources_file, &final_xml)?;

        glib_build_tools::compile_resources(
            &self.source_directories,
            self.paths
                .output_resources_file
                .to_str()
                .context("Invalid XML path")?,
            self.paths
                .output_resources_compiled_file
                .to_str()
                .context("Invalid compiled file path")?,
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

// ===== CARGO ENVIRONMENT VARIABLES =====

struct CargoEnvironmentVariables;

impl CargoEnvironmentVariables {
    fn emit_build_configuration_flags() {
        println!("cargo:rustc-cfg=runtime");

        if std::env::var("SCHEMAS_INSTALLED")
            .map(|v| v == "1")
            .unwrap_or(false)
        {
            println!("cargo:rustc-cfg=schemas_installed");
        }
    }

    fn emit_application_metadata(metadata: &Metadata, paths: &Paths, resources_path: &Path) {
        println!("cargo:rustc-env=APP_NAME={}", metadata.name);
        println!("cargo:rustc-env=APP_DESCRIPTION={}", metadata.description);
        println!("cargo:rustc-env=APP_VERSION={}", metadata.version);
        println!("cargo:rustc-env=APP_ID={}", metadata.id);
        println!("cargo:rustc-env=APP_PREFIX={}", metadata.prefix);
        println!("cargo:rustc-env=APP_TITLE={}", metadata.title);
        println!("cargo:rustc-env=APP_AUTHORS={}", metadata.authors.join(","));
        println!("cargo:rustc-env=APP_RESOURCES={}", resources_path.display());
        println!(
            "cargo:rustc-env=APP_CATALOG={}",
            paths.output_catalog_file.display()
        );

        if let Ok(dir) = std::env::var("GSETTINGS_SCHEMA_DIR") {
            println!("cargo:rustc-env=GSETTINGS_SCHEMA_DIR={dir}");
        }
    }

    fn emit_all_environment_variables(metadata: &Metadata, paths: &Paths, resources_path: &Path) {
        Self::emit_build_configuration_flags();
        Self::emit_application_metadata(metadata, paths, resources_path);
    }
}

// ===== STYLE PROCESSOR =====

struct StyleProcessor<'a> {
    paths: &'a Paths,
}

impl<'a> StyleProcessor<'a> {
    const fn new(paths: &'a Paths) -> Self {
        Self { paths }
    }

    fn check_style_updated(&self) -> Result<bool> {
        FileSystemHelper::target_exists_and_is_newer(
            &self.paths.style_file,
            &self.paths.output_resources_compiled_file,
        )
    }
}

// ===== BUILD PIPELINE =====

struct BuildPipeline {
    paths: Paths,
    metadata: Metadata,
}

impl BuildPipeline {
    fn new() -> Result<Self> {
        let metadata = Metadata::extract_from_cargo()?;
        let paths = Paths::new();
        Ok(Self { paths, metadata })
    }

    fn execute_complete_build(&self) -> Result<()> {
        Self::setup_build_environment();

        let icons = self.process_catalog()?;
        let (icons_xml_content, icons_regenerated) = self.process_icons(&icons)?;
        let (templates_xml_content, templates_regenerated) = self.process_templates()?;
        let style_updated = self.process_styles()?;

        #[cfg(target_os = "windows")]
        self.compile_winres()?;

        if Self::should_compile_resources(icons_regenerated, templates_regenerated, style_updated) {
            self.compile_resources(&templates_xml_content, &icons_xml_content)?;
        }

        self.emit_environment_variables();

        Ok(())
    }

    fn setup_build_environment() {
        println!("cargo:rustc-check-cfg=cfg(runtime)");
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=resources");
    }

    fn process_catalog(&self) -> Result<Vec<Icon>> {
        let (icons, _) = CatalogProcessor::new(&self.paths).process_catalog_data()?;
        Ok(icons)
    }

    fn process_icons(&self, icons: &[Icon]) -> Result<(String, bool)> {
        IconProcessor::new(&self.paths).process_icons(icons)
    }

    fn process_templates(&self) -> Result<(String, bool)> {
        TemplateProcessor::new(&self.paths).process_templates()
    }

    fn process_styles(&self) -> Result<bool> {
        StyleProcessor::new(&self.paths).check_style_updated()
    }

    const fn should_compile_resources(
        icons_regenerated: bool,
        templates_regenerated: bool,
        style_updated: bool,
    ) -> bool {
        icons_regenerated || templates_regenerated || style_updated
    }

    fn compile_resources(&self, templates_xml: &str, icons_xml: &str) -> Result<()> {
        self.create_resource_compiler()
            .add_template_replacement("APP_TEMPLATES", templates_xml)
            .add_template_replacement("APP_ICONS", icons_xml)
            .add_source_directory(&self.paths.output_dir)
            .add_source_directory(&self.paths.data_dir)
            .add_source_directory(&self.paths.output_icons_dir)
            .compile_resources()
    }

    #[cfg(target_os = "windows")]
    fn compile_winres(&self) -> Result<&Self> {
        winres::WindowsResource::new()
            .set_icon(self.paths.icon_file.to_str().unwrap())
            .compile()?;

        Ok(self)
    }

    fn create_resource_compiler(&self) -> ResourceCompiler<'_> {
        ResourceCompiler::new(&self.paths, &self.metadata.prefix)
    }

    fn emit_environment_variables(&self) {
        CargoEnvironmentVariables::emit_all_environment_variables(
            &self.metadata,
            &self.paths,
            &self.paths.output_resources_compiled_file,
        );
    }
}

// ===== MAIN =====

fn main() -> Result<()> {
    BuildPipeline::new()?.execute_complete_build()
}
