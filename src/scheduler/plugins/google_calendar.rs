use super::{CanBreak, Plugin};

use std::net::TcpListener;

use google_calendar3::CalendarHub;
use google_calendar3::Channel;
use google_calendar3::{Error, Scope};
use yup_oauth2::{
    ApplicationSecret, Authenticator, AuthenticatorDelegate, DefaultAuthenticatorDelegate,
    DiskTokenStorage, Retry,
};

fn get_available_port() -> Option<u16> {
    (8080..65535).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok())
}

pub struct GoogleCalendar {
}

impl GoogleCalendar {
    pub fn new() -> Result<Self, ()> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("break-time").map_err(|xdg_base_dir_err| ())?;
        let google_oauth_token_path = xdg_dirs
            .place_config_file("google-oauth-token").map_err(|io_err| ())?;
        let google_oauth_token_path_string =
            google_oauth_token_path.to_string_lossy().into_owned();
        let disk_token_storage =
            DiskTokenStorage::new(&google_oauth_token_path_string)
                .expect("Couldn't create a file to hold the google oauth token");

        let secret: ApplicationSecret =
            ApplicationSecret {
                client_id: String::from("728095687622-mpib9rmdtck7e8ln9egelnns6na0me08.apps.googleusercontent.com"),
                // It is weird embedding something called a "client_secret" directly in the source
                // code, but it doesn't seem like this needs to be something that is actually kept
                // secret:
                // https://stackoverflow.com/questions/59416326/safely-distribute-oauth-2-0-client-secret-in-desktop-applications-in-python
                client_secret: String::from("mI7MmEnboy8jdYEBjK9rZ2M2"),
                token_uri: "https://oauth2.googleapis.com/token".to_string(),
                auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
                redirect_uris: vec![
                    "http://127.0.0.1".to_string(),
                    "urn:ietf:wg:oauth:2.0:oob".to_string(),
                ],
                ..Default::default()
            };

        let http_client_for_auth = hyper::Client::with_connector(
            hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new()),
        );
        let http_client_for_cal = hyper::Client::with_connector(
            hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new()),
        );

        let first_available_port = get_available_port().ok_or(())?;

        let auth = Authenticator::new(
            &secret,
            DefaultAuthenticatorDelegate,
            http_client_for_auth,
            disk_token_storage,
            Some(yup_oauth2::FlowType::InstalledRedirect(first_available_port.into())),
        );

        let hub = CalendarHub::new(http_client_for_cal, auth);

        Ok(GoogleCalendar {
        })
    }

    fn can_break(&self) -> Result<CanBreak, ()> {
        Ok(CanBreak::Yes)
    }
}

impl Plugin for GoogleCalendar {
    fn can_break_now(&self) -> Result<CanBreak, ()> {
        self.can_break()
    }
}
