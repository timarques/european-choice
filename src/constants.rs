use super::models::{Category, Catalog, Product};

pub const APP_CATALOG: Catalog = include!(concat!(env!("OUT_DIR"), "/catalog.rs"));
pub const APP_RESOURCES: &[u8] = include_bytes!(env!("APP_RESOURCES"));
pub const APP_ID: &str = env!("APP_ID");
pub const APP_NAME: &str = env!("APP_NAME");
pub const APP_VERSION: &str = env!("APP_VERSION");
pub const APP_PREFIX: &str = env!("APP_PREFIX");
pub const APP_UI_RESOURCE: &str = concat!(env!("APP_PREFIX"), "/ui.xml");
pub const APP_TITLE: &str = env!("APP_TITLE");
pub const APP_DESCRIPTION: &str = env!("APP_DESCRIPTION");
pub const APP_AUTHORS: &str = env!("APP_AUTHORS");