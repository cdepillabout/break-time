pub fn get_atom(
    connection: &xcb::Connection,
    name: &str,
) -> Result<xcb::Atom, xcb::GenericError> {
    // TODO: Don't use this, but instead get the reply asyncrhonously...
    Ok(xcb::intern_atom(&connection, false, name)
        .get_reply()?
        .atom())
}

fn main() {
    // let xid = gdk_sys::gdk_x11_window_get_xid(gdk_window.to_glib_none().0);

    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup: xcb::Setup = conn.get_setup();
    let mut roots: xcb::ScreenIterator = setup.roots();
    let screen: xcb::Screen = roots.nth(screen_num as usize).unwrap();
    let root_window: xcb::Window = screen.root();

    println!("Root window: {}", &root_window);

    let query_tree_reply: xcb::QueryTreeReply =
        xcb::xproto::query_tree(&conn, root_window)
            .get_reply()
            .unwrap();

    println!("query tree reply, root window: {}, query_tree_reply parent window: {}\n", query_tree_reply.root(), query_tree_reply.parent());

    let net_wm_name_atom = get_atom(&conn, "_NET_WM_NAME")
        .expect("should be able to get the _NET_WM_NAME atom");
    let utf8_string_atom = get_atom(&conn, "UTF8_STRING")
        .expect("should be able to get the UTF8_STRING atom");

    println!("query tree reply, children:");

    // TODO: Send a bunch of requests and then get the replies...

    for win in query_tree_reply.children() {
        // TODO: It is possible that the window has disappeared since we originally got the list.

        let starting_offset = 0;
        let length_to_get = 1024;

        let prop_name = xcb::xproto::get_property(
            &conn,
            false,
            *win,
            xcb::xproto::ATOM_WM_NAME,
            xcb::xproto::ATOM_STRING,
            starting_offset,
            length_to_get,
        )
        .get_reply()
        .unwrap();
        let title_format = prop_name.format();
        let title_type = prop_name.type_();
        let title_vec = prop_name.value().to_vec();
        let title = String::from_utf8(title_vec.clone())
            .unwrap_or(String::from("(title not UTF8...)"));

        let prop_net_wm_name = xcb::xproto::get_property(
            &conn,
            false,
            *win,
            net_wm_name_atom,
            utf8_string_atom,
            starting_offset,
            length_to_get,
        )
        .get_reply()
        .unwrap();
        let net_wm_name_format = prop_net_wm_name.format();
        let net_wm_name_type = prop_net_wm_name.type_();
        let net_wm_name_vec = prop_net_wm_name.value().to_vec();
        let net_wm_name = String::from_utf8(net_wm_name_vec.clone())
            .unwrap_or(String::from("(net_wm_name not UTF8...)"));

        let prop_class = xcb::xproto::get_property(
            &conn,
            false,
            *win,
            xcb::xproto::ATOM_WM_CLASS,
            xcb::xproto::ATOM_STRING,
            starting_offset,
            length_to_get,
        )
        .get_reply()
        .unwrap();
        let class_format = prop_class.format();
        let class_type = prop_class.type_();
        let class_all = prop_class.value::<u8>();
        let option_class_index = class_all.iter().position(|&b| b == 0);
        let class_name: String;
        let class: String;

        let prop_trans = xcb::xproto::get_property(
            &conn,
            false,
            *win,
            xcb::xproto::ATOM_WM_TRANSIENT_FOR,
            xcb::xproto::ATOM_WINDOW,
            starting_offset,
            length_to_get,
        )
        .get_reply()
        .unwrap();

        let trans_for_wins: Vec<xcb::Window> = prop_trans.value().to_vec();

        match option_class_index {
            Some(class_index) => {
                class_name =
                    String::from_utf8(class_all[0..class_index].to_vec())
                        .unwrap_or(String::from("(class name not UTF8...)"));
                class = String::from_utf8(
                    class_all[class_index + 1..class_all.len() - 1].to_vec(),
                )
                .unwrap_or(String::from("(class not UTF8...)"));
            }
            None => {
                class_name = String::from("(no class name...)");
                class = String::from("(no class...)");
            }
        }

        // TODO: Still not able to get title correctly for some reason for things like firefox and
        // chrome.
        //
        // Maybe I need to make sure I don't include the final \0 in the title????
        // if class_name == "Navigator" {
        println!("\tchild: {}\n\t\ttrans for wins: {:?}\n\t\tclass: {}\n\t\tclass name: {}\n\t\tclass_format: {}\n\t\tclass_type: {}\n\t\ttitle_format: {}\n\t\ttitle_type: {}\n\t\tnet_wm_name_format: {}\n\t\tnet_wm_name_type: {}\n\t\ttitle: {}\n\t\tnet_wm_name: {}\n\t\ttitle_vec:       {:?}\n\t\tnet_wm_name_vec: {:?}", win, trans_for_wins, &class, &class_name, class_format, class_type, title_format, title_type, net_wm_name_format, net_wm_name_type, title, net_wm_name, title_vec, net_wm_name_vec);
        // }
    }
}
