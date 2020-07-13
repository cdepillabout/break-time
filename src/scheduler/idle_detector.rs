use std::sync::mpsc::Sender;
use std::time::Duration;

pub struct IdleDetector {
    conn: xcb::Connection,
    root_window: xcb::Window,
    restart_wait_time_sender: Sender<()>,
}

impl IdleDetector {
    pub fn new(restart_wait_time_sender: Sender<()>) -> Self {
        let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
        let setup: xcb::Setup = conn.get_setup();
        let mut roots: xcb::ScreenIterator = setup.roots();
        let screen: xcb::Screen = roots.nth(screen_num as usize).unwrap();
        let root_window: xcb::Window = screen.root();

        Self {
            conn,
            root_window,
            restart_wait_time_sender,
        }
    }

    pub fn run(restart_wait_time_sender: Sender<()>) -> ! {
        let idle_detector = Self::new(restart_wait_time_sender);
        loop {
            std::thread::sleep(Duration::from_secs(20));

            let idle_query_res = xcb::screensaver::query_info(&idle_detector.conn, idle_detector.root_window)
                .get_reply()
                .unwrap();

            println!(
                "state: {}, ms_since_user_input: {}, kind: {}",
                idle_query_res.state(),
                idle_query_res.ms_since_user_input(),
                idle_query_res.kind()
            );

        }
    }
}
