use reqwest::Client;

lazy_static! {
    // The Client holds a connection pool internally, so it is advised that you create one and reuse it.
    pub static ref HTTP_CLIENT: Client = Client::new();
}