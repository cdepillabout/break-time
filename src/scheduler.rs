
mod plugins;

use plugins::Plugin;

pub struct Scheduler {
    plugins: Vec<Box<dyn Plugin>>,
}

impl Scheduler {
    pub fn new() -> Self {
        let window_title_plugin = plugins::WindowTitles::new();
        Scheduler {
            plugins: vec![Box::new(window_title_plugin)],
        }
    }
}
