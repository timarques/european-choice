use glib::GString;

use crate::prelude::*;
use std::sync::OnceLock;
use std::cell::Cell;

const STATE_CHANGED_SIGNAL: &str = "state-changed";
const ACTIVE_CSS_CLASS: &str = "active";

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "SidebarSearchRowState")]
pub enum SidebarSearchRowState {
    Active,
    Inactive,
    #[default]
    Idle
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/pt/timarques/european_choice/sidebar_search_row.ui")]
    pub struct SidebarSearchRow {
        #[template_child(id = "sidebar-search-row-entry")]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child(id = "sidebar-search-row-click-gesture")]
        pub click_gesture: TemplateChild<gtk::GestureClick>,

        pub had_content: Cell<bool>,
        pub state: Cell<SidebarSearchRowState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarSearchRow {
        const NAME: &'static str = "SidebarSearchRow";
        type Type = super::SidebarSearchRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
            Self::Type::ensure_type();
        }

        fn instance_init(object: &glib::subclass::InitializingObject<Self>) {
            object.init_template();
        }
    }

    impl ObjectImpl for SidebarSearchRow {
        fn constructed(&self) {
            self.parent_constructed();
            self.had_content.set(false);
            let obj = self.obj();
            obj.setup_search_entry();
            obj.setup_state_changes();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<[glib::subclass::Signal; 1]> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                [
                    glib::subclass::Signal::builder(STATE_CHANGED_SIGNAL)
                        .param_types([<SidebarSearchRowState>::static_type()])
                        .build(),
                ]
            })
        }
    }
    
    impl WidgetImpl for SidebarSearchRow {}
    impl ListBoxRowImpl for SidebarSearchRow {}
}

glib::wrapper! {
    pub struct SidebarSearchRow(ObjectSubclass<imp::SidebarSearchRow>)
        @extends gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl SidebarSearchRow {
    fn setup_search_entry(&self) {
        let imp = self.imp();
        
        let this_weak = self.downgrade();
        imp.search_entry.connect_stop_search(move |_entry| {
            if let Some(this) = this_weak.upgrade() {
                this.set_state(SidebarSearchRowState::Inactive);
                this.clear_search();
            }
        });

        let this_weak = self.downgrade();
        imp.search_entry.connect_search_started(move |_entry| {
            if let Some(this) = this_weak.upgrade() {
                this.set_state(SidebarSearchRowState::Active);
            }
        });

        let this_weak = self.downgrade();
        imp.search_entry.connect_search_changed(move |_entry| {
            if let Some(this) = this_weak.upgrade() {
                this.set_state(SidebarSearchRowState::Active);
            }
        });

        let this_weak = self.downgrade();
        imp.click_gesture.connect_pressed(move |_, _, _, _| {
            if let Some(this) = this_weak.upgrade() {
                this.set_state(SidebarSearchRowState::Active);
            }
        });
    }

    fn setup_state_changes(&self) {
        self.connect_state_changed(|this, state| {
            match state {
                SidebarSearchRowState::Active => {
                    this.imp().search_entry.grab_focus();
                    this.add_css_class(ACTIVE_CSS_CLASS);
                },
                SidebarSearchRowState::Inactive | SidebarSearchRowState::Idle => {
                    this.remove_css_class(ACTIVE_CSS_CLASS);
                }
            }
        });
    }

    fn set_successful_search(&self, is_successful: bool) {
        let search_entry = &self.imp().search_entry;

        search_entry.remove_css_class("success");
        search_entry.remove_css_class("error");

        if self.is_empty() {
            return;
        } 
        
        if is_successful {
            search_entry.add_css_class("success");
        } else {
            search_entry.add_css_class("error");
        }
    }

    pub fn state(&self) -> SidebarSearchRowState {
        self.imp().state.get()
    }

    pub fn set_state(&self, new_state: SidebarSearchRowState) -> bool {
        let current_state = self.imp().state.get();
        let effective_state = match (new_state, self.is_empty()) {
            (SidebarSearchRowState::Inactive, false) => return false,
            (SidebarSearchRowState::Active, true) => {
                self.imp().search_entry.grab_focus();
                SidebarSearchRowState::Idle
            },
            (state, _) => state
        };

        if current_state == effective_state {
            return false;
        }

        self.imp().state.set(effective_state);
        self.emit_by_name::<()>(STATE_CHANGED_SIGNAL, &[&effective_state]);
        true
    }

    pub fn clear_search(&self) {
        self.imp().search_entry.set_text("");
        self.set_state(SidebarSearchRowState::Idle);
    }

    pub fn search_text(&self) -> GString {
        self.imp().search_entry.text()
    }

    pub fn is_empty(&self) -> bool {
        self.imp().search_entry.text().is_empty()
    }

    pub fn connect_search_changed<F>(&self, callback: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, &str) -> bool + 'static,
    {
        let this_weak = self.downgrade();
        self.imp().search_entry.connect_search_changed(move |entry| {
            let has_content = !entry.text().is_empty();
            if 
                let Some(this) = this_weak.upgrade()
                && (this.imp().had_content.get() || has_content)
            {
                this.imp().had_content.set(has_content);
                let is_successful = callback(&this, &entry.text());
                this.set_successful_search(is_successful);
            }
        })
    }

    pub fn connect_state_changed<F>(&self, callback: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, SidebarSearchRowState) + 'static,
    {
        self.connect_local(STATE_CHANGED_SIGNAL, true, move |values| {
            let this = values[0].get::<Self>().unwrap();
            let state = values[1].get::<SidebarSearchRowState>().unwrap();
            callback(&this, state);
            None
        })
    }
}