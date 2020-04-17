use google_calendar3::CalendarHub;
use google_calendar3::Channel;
use google_calendar3::{Error, Result, Scope};

use std::default::Default;
use std::net::TcpListener;

use yup_oauth2::{
    ApplicationSecret, Authenticator, AuthenticatorDelegate, DiskTokenStorage,
    Retry,
};

struct MyAuthDel;

impl AuthenticatorDelegate for MyAuthDel {
    fn connection_error(&mut self, err: &hyper::Error) -> Retry {
        println!("Got connection error: {:?}", err);
        Retry::Abort
    }

    /// Called if the request code is expired. You will have to start over in this case.
    /// This will be the last call the delegate receives.
    /// Given `DateTime` is the expiration date
    // fn expired(&mut self, _: &DateTime<Utc>) {}

    /// Called if the user denied access. You would have to start over.
    /// This will be the last call the delegate receives.
    fn denied(&mut self) {
        println!("User denied access");
    }

    fn token_refresh_failed(
        &mut self,
        error: &String,
        error_description: &Option<String>,
    ) {
        println!(
            "Token refresh failed: {:?} ({:?})",
            error, error_description
        );
    }
}

fn get_available_port() -> Option<u16> {
    (8080..65535).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok())
}

fn main() {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("break-time")
        .expect("Couldn't find the xdg base directory.");
    let google_oauth_token_path = xdg_dirs
        .place_config_file("google-oauth-token")
        .expect("Can't create xdg configuration directory");
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

    let port = get_available_port().expect("Able to get the port...");

    println!("before creating auth...");
    let auth = Authenticator::new(
        &secret,
        MyAuthDel,
        http_client_for_auth,
        disk_token_storage,
        Some(yup_oauth2::FlowType::InstalledRedirect(port.into())),
    );
    println!("after creating auth...");

    let hub = CalendarHub::new(http_client_for_cal, auth);

    println!("after creating hub...");
    // As the method needs a request, you would usually fill it with the desired information
    // into the respective structure. Some of the parts shown here might not be applicable !
    // Values shown here are possibly random and not representative !
    let req = Channel::default();

    println!("after creating channel...");

    let (_, calendar_list_res) = hub
        .calendar_list()
        .list()
        .add_scope(Scope::Readonly)
        .add_scope(Scope::EventReadonly)
        .doit()
        .expect("couldn't get a response from calendar_list");

    let calendars = calendar_list_res
        .items
        .expect("There should be some calendars available");

    let calendar_ids: Vec<&str> = calendars
        .iter()
        .map(|calendar| {
            calendar
                .id
                .as_deref()
                .expect("Calendars should always have ids")
        })
        .collect();

    let now: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    let ten_minutes_ago: chrono::DateTime<chrono::Utc> =
        now - chrono::Duration::minutes(10);
    let in_twenty_mins: chrono::DateTime<chrono::Utc> =
        now + chrono::Duration::minutes(20);
    println!(
        "now: {}, after twenty: {}",
        now.to_rfc3339(),
        in_twenty_mins.to_rfc3339()
    );

    for calendar_id in calendar_ids {
        let result = hub
            .events()
            .list(calendar_id)
            .add_scope(Scope::Readonly)
            .add_scope(Scope::EventReadonly)
            // all events the occur over the next 20 minutes
            .time_min(&ten_minutes_ago.to_rfc3339())
            .time_max(&in_twenty_mins.to_rfc3339())
            .doit();
        print!("\n\nevents for {}: {:?}", calendar_id, result);
    }
}
