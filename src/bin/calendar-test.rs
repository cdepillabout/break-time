
use google_calendar3::CalendarHub;
use google_calendar3::Channel;
use google_calendar3::{Result, Error};

use std::default::Default;
use std::net::TcpListener;

use yup_oauth2::{Authenticator, AuthenticatorDelegate, ApplicationSecret, MemoryStorage, Retry};

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

    fn token_refresh_failed(&mut self, error: &String, error_description: &Option<String>) {
        println!("Token refresh failed: {:?} ({:?})", error, error_description);
    }
}

fn get_available_port() -> Option<u16> {
    (8080..65535).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok())
}

fn main() {
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

    let mut http_client_for_auth = hyper::Client::with_connector(hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new()));
    let mut http_client_for_cal = hyper::Client::with_connector(hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new()));

    println!("before creating auth...");
    let auth = Authenticator::new(&secret, MyAuthDel, http_client_for_auth, MemoryStorage::default(), Some(yup_oauth2::FlowType::InstalledRedirect(8080)));
    println!("after creating auth...");

    let mut hub = CalendarHub::new(http_client_for_cal, auth);

    println!("after creating hub...");
    // As the method needs a request, you would usually fill it with the desired information
    // into the respective structure. Some of the parts shown here might not be applicable !
    // Values shown here are possibly random and not representative !
    let mut req = Channel::default();

    println!("after creating channel...");

    // You can configure optional parameters by calling the respective setters at will, and
    // execute the final call using `doit()`.
    // Values shown here are possibly random and not representative !
    // let result = hub.events().watch(req, "calendarId")
    //             .updated_min("ea")
    //             .time_zone("no")
    //             .time_min("justo")
    //             .time_max("justo")
    //             .sync_token("et")
    //             .single_events(true)
    //             .show_hidden_invitations(true)
    //             .show_deleted(false)
    //             .add_shared_extended_property("Lorem")
    //             .q("et")
    //             .add_private_extended_property("duo")
    //             .page_token("aliquyam")
    //             .order_by("sea")
    //             .max_results(-55)
    //             .max_attendees(-75)
    //             .i_cal_uid("erat")
    //             .always_include_email(false)
    //             .doit();

    let result = hub.calendar_list().list().doit();

    println!("result: {:?}", result);
}
