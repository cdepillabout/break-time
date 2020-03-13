
use gtk::prelude::*;
use libappindicator::{AppIndicator, AppIndicatorStatus};
use std::io::prelude::*;
use std::path::Path;

static IMG: &'static [u8] = include_bytes!("../../imgs/clock.png");

fn main() {
    gtk::init().unwrap();

    let mut indicator = AppIndicator::new("libappindicator test application", "");

    indicator.set_status(AppIndicatorStatus::Active);

    // let (mut temp_icon_file, temp_icon_file_path) = tempfile::NamedTempFile::new().expect("couldn't create NamedTempFile.").into_parts();
    // temp_icon_file.write_all(IMG);
    // app.set_icon_from_file(temp_icon_file_path.to_str().expect("temp file path is not utf8"))?;

    // let icon_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    // indicator.set_icon_theme_path(icon_path.to_str().unwrap());
    // let (mut temp_icon_file, temp_icon_file_path) = tempfile::NamedTempFile::new().expect("couldn't create NamedTempFile.").into_parts();
    // temp_icon_file.write_all(IMG);
    // let temp_icon_dir = temp_icon_file_path.parent().expect("temp icon file should be able to get parent");
    // let temp_icon_file_name = temp_icon_file_path.file_name().expect("temp icon file should have a file name"); //.expect("temp icon file should be able to get parent");
    // println!("icon file path: {:?}, icon dir: {:?}, icon name: {:?}", temp_icon_file_path, temp_icon_dir, temp_icon_file_name);
    // indicator.set_icon_theme_path(&temp_icon_dir.to_string_lossy());
    // indicator.set_icon_full(&temp_icon_file_name.to_string_lossy(), "icon");

    let icon_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("imgs");
    indicator.set_icon_theme_path(icon_path.to_str().unwrap());
    indicator.set_icon_full("clock", "icon");

    indicator.set_label("This is a label for the indicator", "this is a guide for the indicator (dunno)");
    indicator.set_title("This is a title for the indicator");

    let mut m = gtk::Menu::new();
    let mi = gtk::CheckMenuItem::new_with_label("Hello RUST");
    mi.connect_activate(|_| {
        gtk::main_quit();
    });
    m.append(&mi);
    indicator.set_menu(&mut m);

    m.show_all();

    gtk::main();
}
