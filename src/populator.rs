use super::repository::Repository;
use super::models::Category;
use super::ui::Ui;
use super::widgets::{
    OverviewProductGroupWidget,
    OverviewProductRowWidget,
    SidebarCountryItemWidget,
    SidebarRowWidget
};

pub struct Populator {}

impl Populator {

    pub fn populate(ui: &Ui, repository: Repository) {
        let categories = repository.categories_sorted();

        Self::populate_sidebar_country_row(ui, repository);
        Self::populate_sidebar_category_list(ui, &categories);
        Self::populate_overview(ui, repository, &categories);
    }

    fn populate_sidebar_country_row(ui: &Ui, repository: Repository) {
        let country_row = ui.country_row();
        for country in Repository::countries_sorted() {
            if repository
                .product_indices_by_country(country)
                .is_some_and(|indices| !indices.is_empty())
            {
                let item = SidebarCountryItemWidget::from_country(country);
                country_row.add_item(&item);
            }
        }
    }

    fn populate_sidebar_category_list(ui: &Ui, categories: &[(usize, &Category)]) {
        let category_list = ui.category_list();
        for (index, category) in categories {
            let row = SidebarRowWidget::from_category(category, *index);
            category_list.append_row(row);
        }
    }

    fn populate_overview(ui: &Ui, repository: Repository, categories: &[(usize, &Category)]) {
        for (category_index, category) in categories {
            if let Some(products_indices) = repository.category_products_sorted(category) {
                let group = OverviewProductGroupWidget::from_category(category, *category_index);

                for (product_index, product) in products_indices {
                    let row = OverviewProductRowWidget::from_product(product, product_index);
                    group.append_row(row);
                }

                ui.overview_page().add_group(group);
            }
        }
    }
}