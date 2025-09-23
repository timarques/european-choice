use super::super::prelude::*;
use super::sidebar_country_item::SidebarCountryItem;

use std::cell::Cell;
use std::sync::OnceLock;
use std::collections::HashMap;
use std::cell::RefCell;

const DEFAULT_INDEX: u32 = 0;
const STATE_CHANGED_SIGNAL: &str = "state-changed";

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "SidebarCountryRowState")]
pub enum SidebarCountryRowState {
    Active,
    #[default]
    Inactive
}

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate)]
    #[template(resource = "/pt/timarques/european_choice/sidebar_country_row.ui")]
    pub struct SidebarCountryRow {
        #[template_child(id = "sidebar-country-row-drop-down")]
        pub dropdown: TemplateChild<gtk::DropDown>,

        pub default_item: SidebarCountryItem,
        pub state: Cell<SidebarCountryRowState>,
        pub list_store: gtk::gio::ListStore,
        pub factory: gtk::SignalListItemFactory,
        pub map: RefCell<HashMap<usize, u32>>
    }

    impl Default for SidebarCountryRow {
        fn default() -> Self {
            let default_item = SidebarCountryItem::new("All", "Countries", None);
            default_item.set_index(DEFAULT_INDEX);

            Self {
                state: Cell::new(SidebarCountryRowState::Inactive),
                dropdown: TemplateChild::default(),
                list_store: gtk::gio::ListStore::new::<SidebarCountryItem>(),
                factory: gtk::SignalListItemFactory::new(),
                default_item,
                map: RefCell::new(HashMap::new())
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarCountryRow {
        const NAME: &'static str = "SidebarCountryRow";
        type Type = super::SidebarCountryRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(class: &mut Self::Class) {
            Self::bind_template(class);
            Self::Type::ensure_type();
        }

        fn instance_init(initializing_object: &glib::subclass::InitializingObject<Self>) {
            initializing_object.init_template();
        }
    }

    impl ObjectImpl for SidebarCountryRow {
        fn constructed(&self) {
            self.parent_constructed();
            
            self.obj().setup_dropdown();
            self.obj().setup_factory();
            self.obj().setup_state_changes();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<[glib::subclass::Signal; 1]> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                [
                    glib::subclass::Signal::builder(STATE_CHANGED_SIGNAL)
                        .param_types([<SidebarCountryRowState>::static_type()])
                        .build(),
                ]
            })
        }
    }
    
    impl WidgetImpl for SidebarCountryRow {}
    impl ListBoxRowImpl for SidebarCountryRow {}
}

glib::wrapper! {
    pub struct SidebarCountryRow(ObjectSubclass<imp::SidebarCountryRow>)
        @extends gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl SidebarCountryRow {

    fn setup_dropdown(&self) {
        let imp = self.imp();
        imp.list_store.append(&imp.default_item);
        imp.dropdown.set_model(Some(&imp.list_store));
        imp.dropdown.set_factory(Some(&imp.factory));

        let this_weak = self.downgrade();
        imp.dropdown.connect_selected_item_notify(move |_| {
            if let Some(this) = this_weak.upgrade()
            {
                let new_state = if this.is_selected_default() {
                    SidebarCountryRowState::Inactive
                } else {
                    SidebarCountryRowState::Active
                };

                this.set_state(new_state);
            }
        });
    }

    fn setup_factory(&self) {
        let factory = &self.imp().factory;

        factory.connect_setup(|_factory, list_item| {
            if let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() {
                let country_item = SidebarCountryItem::new("", "", None);
                list_item.set_child(Some(&country_item));
            }
        });

        factory.connect_bind(move |_factory, list_item| {
            if let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>()
                && let Some(data_item) = list_item.item().and_downcast_ref::<SidebarCountryItem>()
            {
                if let Some(widget_child) = list_item.child().and_then(|widget| widget.downcast::<SidebarCountryItem>().ok()) {
                    widget_child.set_label(data_item.label());
                    widget_child.set_index(data_item.index());
                    widget_child.set_caption(data_item.caption());
                    widget_child.set_caption_visible(false);
                    if let Some(flag) = data_item.flag() {
                        widget_child.set_flag(flag);
                    }
                } else {
                    data_item.set_caption_visible(true);
                    list_item.set_child(Some(data_item));
                }
            }
        });

        factory.connect_unbind(|_factory, list_item| {
            if let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() {
                list_item.set_child(gtk::Widget::NONE);
            }
        });
    }

    fn setup_state_changes(&self) {
        self.connect_state_changed(|this, state| {
            match state {
                SidebarCountryRowState::Active => this.add_css_class("active"),
                SidebarCountryRowState::Inactive => this.remove_css_class("active")
            }
        });
    }

    pub fn is_selected_default(&self) -> bool {
        self.imp().dropdown.selected() == DEFAULT_INDEX
    }

    pub fn set_state(&self, new_state: SidebarCountryRowState) -> bool {
        let previous_state = self.imp().state.get();
        if previous_state == new_state {
            return false;
        }

        match (new_state, self.is_selected_default()) {
            (SidebarCountryRowState::Active, true) => {
                self.imp().dropdown.emit_activate();
                return false;
            },
            (SidebarCountryRowState::Inactive, false) => return false,
            _ => ()
        }

        self.imp().state.set(new_state);
        self.emit_by_name::<()>("state-changed", &[&new_state]);
        true
    }

    pub fn add_item(&self, item: &SidebarCountryItem) {
        let imp = self.imp();
        let items_count = imp.list_store.n_items();
        imp.list_store.append(item);

        if items_count == 1 {
            imp.dropdown.set_selected(items_count);
            imp.dropdown.set_selected(DEFAULT_INDEX);
        }

        imp.map.borrow_mut().insert(item.index() as usize, items_count);
    }

    pub fn selected_item(&self) -> Option<SidebarCountryItem> {
        let dropdown = &self.imp().dropdown;
        (dropdown.selected() != DEFAULT_INDEX)
            .then(|| dropdown.selected_item().and_downcast::<SidebarCountryItem>())
            .flatten()
    }

    pub fn select_item_by_index(&self, index: usize) -> bool {
        let imp = self.imp();
        let dropdown = &imp.dropdown;
        imp.map
            .borrow()
            .get(&index)
            .copied()
            .is_some_and(|position| {
                dropdown.set_selected(position);
                true
            })
    }

    pub fn select_default_item(&self) {
        self.imp().dropdown.set_selected(DEFAULT_INDEX);
    }

    pub fn connect_state_changed<F>(&self, callback: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, SidebarCountryRowState) + 'static,
    {
        self.connect_local(STATE_CHANGED_SIGNAL, true, move |values| {
            let this = values[0].get::<Self>().unwrap();
            let state = values[1].get::<SidebarCountryRowState>().unwrap();
            callback(&this, state);
            None
        })
    }

    pub fn connect_item_selected<F>(&self, callback: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, Option<&SidebarCountryItem>) + 'static
    {
        let this_weak = self.downgrade();
        self.imp().dropdown.connect_selected_item_notify(move |_| {
            if
                let Some(this) = this_weak.upgrade()
            {
                let item = this.selected_item();
                callback(&this, item.as_ref());
            }
        })
    }
}