
use std::io::prelude::*;

static IMG: &'static [u8] = include_bytes!("../../imgs/clock.png");

fn main() -> Result<(),systray::Error> {

    let app = systray::Application::new().expect("couldn't create systray application");
    // w.set_icon_from_file(&"C:\\Users\\qdot\\code\\git-projects\\systray-rs\\resources\\rust.ico".to_string());
    // w.set_tooltip(&"Whatever".to_string());

    let (mut temp_icon_file, temp_icon_file_path) = tempfile::NamedTempFile::new().expect("couldn't create NamedTempFile.").into_parts();
    temp_icon_file.write_all(IMG);
    app.set_icon_from_file(temp_icon_file_path.to_str().expect("temp file path is not utf8"))?;

    let ten_secs = std::time::Duration::from_secs(10);
    std::thread::sleep(ten_secs);

    Ok(())
}
