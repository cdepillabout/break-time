use super::{CanBreak, Plugin};

use crate::config::Config;
use crate::platform::create_display;
use crate::platform::display::{DisplayBackend, WindowInfo};

pub struct WindowTitles {
    display: Box<dyn DisplayBackend>,
}

impl WindowTitles {
    pub fn new(_config: &Config) -> Self {
        Self {
            display: create_display(),
        }
    }

    fn can_break(&self) -> Result<CanBreak, ()> {
        let all_win_props: Vec<WindowInfo> = self.display.list_windows()?;
        let all_can_break_preds = CanBreakPreds::all();
        let can_break_bool = all_win_props.iter().all(|win_props| {
            all_can_break_preds.can_break(win_props).into_bool()
        });
        let can_break_res = CanBreak::from_bool(can_break_bool);
        Ok(can_break_res)
    }
}

struct CanBreakPred<F>(F);

impl CanBreakPred<Box<dyn Fn(&WindowInfo) -> CanBreak>> {
    fn from_name_class<G>(g: G) -> Self
    where
        G: 'static + Fn(&str, &str, &str) -> CanBreak,
    {
        Self(Box::new(move |win_props: &WindowInfo| {
            match (
                &win_props.net_wm_name,
                &win_props.class_name,
                &win_props.class,
            ) {
                (Ok(net_wm_name), Ok(class_name), Ok(class)) => {
                    g(net_wm_name, class_name, class)
                }
                _ => CanBreak::Yes,
            }
        }))
    }

    fn can_break(&self, win_props: &WindowInfo) -> CanBreak {
        self.0(win_props)
    }
}

struct CanBreakPreds<F>(Vec<CanBreakPred<F>>);

impl CanBreakPreds<Box<dyn Fn(&WindowInfo) -> CanBreak>> {
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

    fn can_break(&self, win_props: &WindowInfo) -> CanBreak {
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

impl Plugin for WindowTitles {
    fn name(&self) -> &'static str {
        "window_titles"
    }

    fn can_break_now(&self) -> Result<CanBreak, Box<dyn std::error::Error>> {
        let custom_error =
            std::io::Error::other("could not enumerate X11 windows");
        self.can_break()
            .map_err(|()| Box::new(custom_error) as Box<dyn std::error::Error>)
    }
}
