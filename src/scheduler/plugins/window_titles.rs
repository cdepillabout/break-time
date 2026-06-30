use super::{CanBreak, Plugin};

use crate::config::Config;
use crate::x11::X11;

use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt as _, Window};

pub struct WindowTitles {
    x11: X11,
    net_wm_name_atom: Atom,
    utf8_string_atom: Atom,
}

impl WindowTitles {
    pub fn new(_config: &Config) -> Result<Self, ()> {
        let x11 = X11::connect();

        let net_wm_name_atom = x11.create_atom("_NET_WM_NAME").ok_or(())?;
        let utf8_string_atom = x11.create_atom("UTF8_STRING").ok_or(())?;

        Ok(Self {
            x11,
            net_wm_name_atom,
            utf8_string_atom,
        })
    }

    fn get_string_prop(
        &self,
        win: Window,
        property: Atom,
        type_: Atom,
    ) -> Result<String, ()> {
        let reply = self
            .x11
            .conn
            .get_property(
                false,
                win,
                property,
                type_,
                PROP_STARTING_OFFSET,
                PROP_LENGTH_TO_GET,
            )
            .map_err(|_| ())?
            .reply()
            .map_err(|_| ())?;
        String::from_utf8(reply.value).map_err(|_| ())
    }

    fn get_class_info(&self, win: Window) -> ClassInfo<()> {
        let res_value = self
            .x11
            .conn
            .get_property(
                false,
                win,
                AtomEnum::WM_CLASS,
                AtomEnum::STRING,
                PROP_STARTING_OFFSET,
                PROP_LENGTH_TO_GET,
            )
            .map_err(|_| ())
            .and_then(|cookie| cookie.reply().map_err(|_| ()));

        match res_value {
            Err(()) => ClassInfo::err(()),
            Ok(reply) => ClassInfo::from_raw_data(&reply.value, (), |_| ()),
        }
    }

    fn get_transient_for(&self, win: Window) -> Result<Vec<Window>, ()> {
        let reply = self
            .x11
            .conn
            .get_property(
                false,
                win,
                AtomEnum::WM_TRANSIENT_FOR,
                AtomEnum::WINDOW,
                PROP_STARTING_OFFSET,
                PROP_LENGTH_TO_GET,
            )
            .map_err(|_| ())?
            .reply()
            .map_err(|_| ())?;
        Ok(reply.value32().map(Iterator::collect).unwrap_or_default())
    }

    fn get_win_props(&self, win: Window) -> WinProps {
        let wm_name = self.get_string_prop(
            win,
            AtomEnum::WM_NAME.into(),
            AtomEnum::STRING.into(),
        );
        let net_wm_name = self.get_string_prop(
            win,
            self.net_wm_name_atom,
            self.utf8_string_atom,
        );
        let transient_for_wins = self.get_transient_for(win);
        let ClassInfo {
            name: class_name,
            class,
        } = self.get_class_info(win);

        WinProps {
            wm_name,
            net_wm_name,
            transient_for_wins,
            class_name,
            class,
        }
    }

    fn get_all_win_props(&self) -> Result<Vec<WinProps>, ()> {
        let wins = self.get_all_wins()?;
        Ok(wins.iter().map(|win| self.get_win_props(*win)).collect())
    }

    fn get_root_win(&self) -> Result<Window, ()> {
        self.x11.get_root_win().ok_or(())
    }

    fn get_all_wins(&self) -> Result<Vec<Window>, ()> {
        let root_win = self.get_root_win()?;

        let query_tree_reply = self
            .x11
            .conn
            .query_tree(root_win)
            .map_err(|_| ())?
            .reply()
            .map_err(|_| ())?;

        Ok(query_tree_reply.children)
    }

    fn can_break(&self) -> Result<CanBreak, ()> {
        let all_win_props: Vec<WinProps> = self.get_all_win_props()?;
        let all_can_break_preds = CanBreakPreds::all();
        let can_break_bool = all_win_props.iter().all(|win_props| {
            all_can_break_preds.can_break(win_props).into_bool()
        });
        let can_break_res = CanBreak::from_bool(can_break_bool);
        Ok(can_break_res)
    }
}

struct CanBreakPred<F>(F);

impl CanBreakPred<Box<dyn Fn(&WinProps) -> CanBreak>> {
    fn from_name_class<G>(g: G) -> Self
    where
        G: 'static + Fn(&str, &str, &str) -> CanBreak,
    {
        Self(Box::new(move |win_props: &WinProps| {
            match (
                &win_props.net_wm_name,
                &win_props.class_name,
                &win_props.class,
            ) {
                (Ok(net_wm_name), Ok(class_name), Ok(class)) => {
                    g(&net_wm_name, &class_name, &class)
                }
                _ => CanBreak::Yes,
            }
        }))
    }

    fn can_break(&self, win_props: &WinProps) -> CanBreak {
        self.0(win_props)
    }
}

struct CanBreakPreds<F>(Vec<CanBreakPred<F>>);

impl CanBreakPreds<Box<dyn Fn(&WinProps) -> CanBreak>> {
    fn all() -> Self {
        Self(vec![
            // BigBlueButton in browser
            CanBreakPred::from_name_class(
                |net_wm_name: &str,
                 class_name: &str,
                 class: &str|
                 -> CanBreak {
                    browser_title_starts_with(
                        class,
                        class_name,
                        net_wm_name,
                        "BigBlueButton",
                    )
                },
            ),
            // Google Meet in browser
            CanBreakPred::from_name_class(
                |net_wm_name: &str,
                 class_name: &str,
                 class: &str|
                 -> CanBreak {
                    browser_title_starts_with(
                        class,
                        class_name,
                        net_wm_name,
                        "Meet",
                    )
                },
            ),
            // Jitsi in browser
            CanBreakPred::from_name_class(
                |net_wm_name: &str,
                 class_name: &str,
                 class: &str|
                 -> CanBreak {
                    browser_title_contains(
                        class,
                        class_name,
                        net_wm_name,
                        "Jitsi Meet",
                    )
                },
            ),
            // Slack: Initiating a Slack call in browser
            CanBreakPred::from_name_class(
                |net_wm_name: &str,
                 class_name: &str,
                 class: &str|
                 -> CanBreak {
                    browser_title_starts_with(
                        class,
                        class_name,
                        net_wm_name,
                        "Slack | Calling ",
                    )
                },
            ),
            // Slack: In a Slack call in browser
            CanBreakPred::from_name_class(
                |net_wm_name: &str,
                 class_name: &str,
                 class: &str|
                 -> CanBreak {
                    browser_title_starts_with(
                        class,
                        class_name,
                        net_wm_name,
                        "Slack | Slack call ",
                    )
                },
            ),
            // Skype
            CanBreakPred::from_name_class(
                |net_wm_name: &str,
                 class_name: &str,
                 class: &str|
                 -> CanBreak {
                    if class == "Skype"
                        && class_name == "skype"
                        && net_wm_name == "Skype"
                    {
                        CanBreak::No
                    } else {
                        CanBreak::Yes
                    }
                },
            ),
            // Zoom
            CanBreakPred::from_name_class(
                |net_wm_name: &str,
                 class_name: &str,
                 class: &str|
                 -> CanBreak {
                    if class.contains("zoom")
                        && class_name.contains("zoom")
                        && net_wm_name.contains("Zoom")
                    {
                        CanBreak::No
                    } else {
                        CanBreak::Yes
                    }
                },
            ),
        ])
    }

    fn can_break(&self, win_props: &WinProps) -> CanBreak {
        CanBreak::from_bool(self.0.iter().all(|can_break_pred| {
            can_break_pred.can_break(win_props).into_bool()
        }))
    }
}

fn is_browser(class: &str, class_name: &str) -> bool {
    (class == "Chromium-browser" && class_name == "chromium-browser")
        || (class == "Chromium" && class_name == "chromium")
        || (class == "Firefox" && class_name == "Navigator")
        || (class == "firefox" && class_name == "Navigator")
        || (class == "firefox_firefox" && class_name == "Navigator")
        || (class == "Firefox" && class_name == "firefox")
}

fn browser_title_starts_with_raw(
    class: &str,
    class_name: &str,
    net_wm_name: &str,
    title_starts_with: &str,
) -> bool {
    is_browser(class, class_name) && net_wm_name.starts_with(title_starts_with)
}

fn browser_title_starts_with(
    class: &str,
    class_name: &str,
    net_wm_name: &str,
    title_starts_with: &str,
) -> CanBreak {
    if browser_title_starts_with_raw(
        class,
        class_name,
        net_wm_name,
        title_starts_with,
    ) {
        CanBreak::No
    } else {
        CanBreak::Yes
    }
}

fn browser_title_contains_raw(
    class: &str,
    class_name: &str,
    net_wm_name: &str,
    title_contains: &str,
) -> bool {
    is_browser(class, class_name) && net_wm_name.contains(title_contains)
}

fn browser_title_contains(
    class: &str,
    class_name: &str,
    net_wm_name: &str,
    title_contains: &str,
) -> CanBreak {
    if browser_title_contains_raw(
        class,
        class_name,
        net_wm_name,
        title_contains,
    ) {
        CanBreak::No
    } else {
        CanBreak::Yes
    }
}

const PROP_STARTING_OFFSET: u32 = 0;
const PROP_LENGTH_TO_GET: u32 = 2048;

#[derive(Clone, Debug)]
struct WinProps {
    #[allow(dead_code)]
    wm_name: Result<String, ()>,
    net_wm_name: Result<String, ()>,
    #[allow(dead_code)]
    transient_for_wins: Result<Vec<Window>, ()>,
    class_name: Result<String, ()>,
    class: Result<String, ()>,
}

struct ClassInfo<T> {
    name: Result<String, T>,
    class: Result<String, T>,
}

impl<T: Clone> ClassInfo<T> {
    fn err(t: T) -> Self {
        Self {
            name: Err(t.clone()),
            class: Err(t),
        }
    }

    fn from_raw_data_with_index<F: Fn(std::string::FromUtf8Error) -> T>(
        raw: &[u8],
        index: usize,
        utf8_err_mapper: F,
    ) -> Self {
        Self {
            name: String::from_utf8(raw[0..index].to_vec())
                .map_err(&utf8_err_mapper),
            class: String::from_utf8(raw[index + 1..raw.len() - 1].to_vec())
                .map_err(utf8_err_mapper),
        }
    }

    fn from_raw_data<F: Fn(std::string::FromUtf8Error) -> T>(
        raw: &[u8],
        no_index_err: T,
        utf8_err_mapper: F,
    ) -> Self {
        let option_index = raw.iter().position(|&b| b == 0);
        match option_index {
            None => Self::err(no_index_err),
            Some(index) => {
                Self::from_raw_data_with_index(raw, index, utf8_err_mapper)
            }
        }
    }
}

impl Plugin for WindowTitles {
    fn can_break_now(&self) -> Result<CanBreak, Box<dyn std::error::Error>> {
        let custom_error =
            std::io::Error::other("TODO: change this to an actual error");
        self.can_break()
            .map_err(|()| Box::new(custom_error) as Box<dyn std::error::Error>)
    }
}
