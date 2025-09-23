use super::super::search_engine::SearchEngine;
use super::super::widgets::{SidebarRowWidget, OverviewProductRowWidget, SidebarSearchRowState};
use super::super::models::Country;
use super::super::ui::Ui;

use std::collections::HashMap;
use std::rc::{Rc, Weak};

struct State {
    ui: Ui,
    engine: SearchEngine
}

pub struct WeakSearch {
    state: Weak<State>
}

impl WeakSearch {
    pub fn upgrade(&self) -> Option<Search> {
        self.state.upgrade().map(|state| Search { state })
    }
}

#[derive(Clone)]
pub struct Search {
    state: Rc<State>
}

impl Search {

    pub fn new(ui: Ui, engine: SearchEngine) -> Self {
        let state = State { ui, engine };
        let this = Self { state: Rc::new(state) };
        this.setup_search_text_changed();
        this.setup_country_selection_changed();
        this
    }

    pub fn activate(&self) -> bool {
        self.state.ui.search_row().set_state(SidebarSearchRowState::Active)
    }

    fn setup_search_text_changed(&self) {
        let this_weak = self.downgrade();
        self.state.ui.search_row().connect_search_changed(move |_, _| {
            this_weak
                .upgrade()
                .is_some_and(|this| this.update_search_results())
        });
    }

    fn setup_country_selection_changed(&self) {
        let this_weak = self.downgrade();
        self.state.ui.country_row().connect_item_selected(move |_, _| {
            if let Some(this) = this_weak.upgrade() {
                this.update_search_results();
            }
        });
    }

    fn update_search_results(&self) -> bool {
        let search_text = self.state.ui.search_row().search_text();
        let country = self.get_selected_country();
        let search_results = self.state.engine.find_by_category(&search_text, country);

        self.update_overview_page(&search_results.by_category);
        self.update_category_list(&search_results.by_category);

        search_results.has_any_matches
    }

    fn get_selected_country(&self) -> Option<Country> {
        self.state.ui
            .country_row()
            .selected_item()
            .and_then(|item| Country::from_index(item.index() as usize))
    }

    fn update_overview_page(&self, results: &[HashMap<usize, bool>]) {
        self.state.ui.overview_page().scroll_to_top();
        self.state.ui.overview_page().groups().iter().for_each(|(_, group)| {
            if let Some(matches) = results.get(group.index() as usize) {
                group.apply_row_filter(|row: &OverviewProductRowWidget| {
                    matches.get(&(row.index() as usize)).copied().unwrap_or(false)
                });
            }
        });
    }

    fn update_category_list(&self, results: &[HashMap<usize, bool>]) {
        self.state.ui.category_list().apply_row_filter(|row: &SidebarRowWidget| {
            results
                .get(row.index() as usize)
                .is_some_and(|matches| matches.values().any(|&v| v))
        });
    }

    pub fn downgrade(&self) -> WeakSearch {
        let state = Rc::downgrade(&self.state);
        WeakSearch { state }
    }
}