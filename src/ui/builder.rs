#[allow(clippy::module_name_repetitions)]
pub trait BuilderExtManualGetObjectExpect {
    fn get_object_expect<T: glib::object::IsA<glib::object::Object>>(
        &self,
        name: &str,
    ) -> T;
}

impl<U> BuilderExtManualGetObjectExpect for U
where
    U: gtk::prelude::BuilderExtManual,
{
    fn get_object_expect<T: glib::object::IsA<glib::object::Object>>(
        &self,
        name: &str,
    ) -> T {
        self.object(name).expect(&format!(
            "Expected to get \"{name}\" from the builder, but failed."
        ))
    }
}

#[must_use]
pub fn create() -> gtk::Builder {
    let glade_src = include_str!("../../glade/ui.glade");
    gtk::Builder::from_string(glade_src)
}
