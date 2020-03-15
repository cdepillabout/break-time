
use super::Plugin;

pub struct WindowTitles {
    conn: xcb::Connection,
    screen_num: i32,
}

impl WindowTitles {
    pub fn new() -> Self {
        let (conn, screen_num) = xcb::Connection::connect(None).unwrap();

        WindowTitles {
            conn,
            screen_num,
        }
        // let setup: xcb::Setup = conn.get_setup();
        // let mut roots: xcb::ScreenIterator = setup.roots();
        // let screen: xcb::Screen = roots.nth(screen_num as usize).unwrap();
        // let root_window: xcb::Window = screen.root();
    }
}

impl Plugin for WindowTitles {
    fn can_break_now(&self) -> bool {
        true
    }
}
