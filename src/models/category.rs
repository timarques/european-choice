use super::String;

#[derive(Debug, Clone)]
pub struct Category {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub summary: String,
    pub icon: String,
}