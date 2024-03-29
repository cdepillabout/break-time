mod default;

use std::default::Default;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::opts::Opts;

use default::DEFAULT_CONFIG_SETTINGS;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(transparent)]
pub struct PluginSettings(pub toml::value::Table);

impl Default for PluginSettings {
    fn default() -> Self {
        let mut google_cal: toml::value::Table = toml::map::Map::new();
        let google_cal_accounts: Vec<String> = vec![];
        let google_cal_accounts_val = toml::Value::try_from(google_cal_accounts)
            .expect(
                "Could not decode google_cal_accounts as toml Value, even though we should be able to."
            );
        google_cal.insert(String::from("accounts"), google_cal_accounts_val);

        let x11_window_title_checker: toml::value::Table =
            toml::map::Map::new();

        let mut plugin_settings_table: toml::value::Table =
            toml::map::Map::new();
        plugin_settings_table.insert(
            String::from("google_calendar"),
            toml::Value::Table(google_cal),
        );
        plugin_settings_table.insert(
            String::from("x11_window_title_checker"),
            toml::Value::Table(x11_window_title_checker),
        );

        Self(plugin_settings_table)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Settings {
    #[serde(default = "default_break_duration_seconds")]
    pub break_duration_seconds: u32,
    #[serde(default = "default_seconds_between_breaks")]
    pub seconds_between_breaks: u32,
    #[serde(default = "default_clicks_to_end_break_early")]
    pub clicks_to_end_break_early: u32,
    #[serde(default = "default_idle_detection_enabled")]
    pub idle_detection_enabled: bool,
    #[serde(default = "default_idle_detection_seconds")]
    pub idle_detection_seconds: u32,
    #[serde(rename = "plugin")]
    pub all_plugin_settings: PluginSettings,
}

const fn default_break_duration_seconds() -> u32 {
    60 * 10
}

const fn default_seconds_between_breaks() -> u32 {
    60 * 50
}

const fn default_clicks_to_end_break_early() -> u32 {
    400
}

const fn default_idle_detection_enabled() -> bool {
    true
}

const fn default_idle_detection_seconds() -> u32 {
    480
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            break_duration_seconds: default_break_duration_seconds(),
            seconds_between_breaks: default_seconds_between_breaks(),
            clicks_to_end_break_early: default_clicks_to_end_break_early(),
            all_plugin_settings: PluginSettings::default(),
            idle_detection_enabled: default_idle_detection_enabled(),
            idle_detection_seconds: default_idle_detection_seconds(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub file_path: PathBuf,
    pub cache_dir: PathBuf,
    pub settings: Settings,
}

impl Config {
    // TODO: Change some of the panics in this function to returning errors.
    pub fn load(opts: &Opts) -> Result<Self, ()> {
        let config_file_name = "config.toml";
        let config_file_path = match &opts.conf_dir {
            Some(conf_dir) => {
                std::fs::create_dir_all(&conf_dir).map_err(|_io_err| ())?;
                conf_dir.join(config_file_name)
            }
            None => {
                let xdg_base_dir =
                    xdg::BaseDirectories::with_prefix("break-time")
                        .map_err(|_xdg_base_dir_err| ())?;
                xdg_base_dir
                    .place_config_file(config_file_name)
                    .map_err(|_io_err| ())?
            }
        };

        let cache_dir = match &opts.cache_dir {
            Some(cache_dir) => cache_dir.clone(),
            None => {
                let xdg_base_dir =
                    xdg::BaseDirectories::with_prefix("break-time")
                        .map_err(|_xdg_base_dir_err| ())?;
                xdg_base_dir.get_cache_home()
            }
        };
        std::fs::create_dir_all(&cache_dir).map_err(|_io_err| ())?;

        // Try reading the config file to see whether it exists or not.
        let res_config_file = std::fs::read_to_string(&config_file_path);
        let settings = match res_config_file {
            // TODO: I should probably check here the reason we are getting an
            // error.  If there is a bad permission on the file, then I
            // should probably just error out fast instead of trying to
            // create a new config file.
            Err(_) => {
                // If we couldn't read the config file, then create a new one
                // from the default.
                let write_res =
                    std::fs::write(&config_file_path, DEFAULT_CONFIG_SETTINGS);
                match write_res {
                    Ok(()) => (),
                    Err(err) =>
                        panic!(
                            "Couldn't write a new config file at {:?} because of the following error: {}",
                            config_file_path,
                            err
                        ),
                }
                Settings::default()
            }
            Ok(config_file) => {
                let res_settings = toml::from_str(&config_file);
                match res_settings {
                    Err(err) => {
                        panic!(
                            "Can't parse config file at {:?} because of the following error: {}",
                            config_file_path,
                            err
                        )
                    }
                    Ok(settings) => settings,
                }
            }
        };

        let config = Self {
            file_path: config_file_path,
            cache_dir,
            settings,
        };

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings_constant_is_same_as_default_impl() {
        let settings_from_default_instance: Settings = Default::default();
        let settings_from_default_const: Settings =
            toml::from_str(DEFAULT_CONFIG_SETTINGS).unwrap();

        assert_eq!(settings_from_default_instance, settings_from_default_const);
    }
}
