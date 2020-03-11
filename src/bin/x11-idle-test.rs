
fn main() {
    // let xid = gdk_sys::gdk_x11_window_get_xid(gdk_window.to_glib_none().0);

    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup: xcb::Setup = conn.get_setup();
    let mut roots: xcb::ScreenIterator = setup.roots();
    let screen: xcb::Screen = roots.nth(screen_num as usize).unwrap();
    let root_window: xcb::Window = screen.root();

    println!("Root window: {}", &root_window);

    let idle_query_res = xcb::screensaver::query_info(&conn, root_window).get_reply().unwrap();

    println!("state: {}, ms_since_user_input: {}, kind: {}", idle_query_res.state(), idle_query_res.ms_since_user_input(), idle_query_res.kind());

}
