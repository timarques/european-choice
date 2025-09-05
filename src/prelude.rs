pub use adw::prelude::*;
pub use adw::subclass::prelude::*;
pub use anyhow::{Result, Context, bail};

pub trait AssertTemplateChildType {
    fn assert_child_type(&self, id: &str);
}

impl <T> AssertTemplateChildType for T
where
    T: ObjectType + glib::translate::FromGlibPtrNone<*mut <T as ObjectType>::GlibType>
{
    fn assert_child_type(&self, id: &str)
    {
        let child_widget = &*self;
        if !child_widget.is::<T>() {
            panic!(
                "Template child with id `{id}` has incompatible type. XML has `{child}`, struct expects `{type}`",
                child = child_widget.type_().name(),
                type = T::static_type().name()
            );
        }
    }
}