//! A library providing a client for the [ProGet](https://inedo.com/proget) API.
//!
//! This library is **heavily** a work in progress, and stability is currently **not guaranteed**.
//! The library also needs a plethora of features to be added still - if there's something you'd
//! like added that's missing, feel free to make an issue or send in a PR on the
//! [GitHub repository](https://github.com/hwittenborn/proget-rust-sdk).
//!
//! Most use cases will involve beginning with a [`Client`]. Please start there
//! if you're trying to find your way around the library.
mod deb;

pub use reqwest;
use reqwest::header::HeaderMap;

/// The user agent we use in requests.
static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// A struct representing a user of a ProGet instance.
pub struct Client {
    http: reqwest::Client,
    server_url: String,
}

impl Client {
    /// Create a new client.
    pub fn new(server_url: &str, api_token: &str) -> Self {
        let mut headers = HeaderMap::new();
        let auth_key = base64::encode(format!("api:{api_token}"));
        let auth_header = format!("Basic {auth_key}");
        headers.insert("Authorization", auth_header.parse().unwrap());

        let http_client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .build()
            .unwrap();

        Self {
            http: http_client,
            server_url: server_url.to_owned(),
        }
    }

    /// Upload a `.deb` package.
    ///
    /// # Arguments
    /// * `feed_name`: The [feed](https://docs.inedo.com/docs/proget-feeds-feed-overview) to upload the `.deb` package to.
    /// * `component_name`: The component in the APT repository to upload the deb to. For example, this would be `component` in `deb https://proget.example.com deb-packages component`.
    /// * `deb_name`: The name of the `.deb` file to register the package under (i.e. `pkg_1.0.0-1_amd64.deb`).
    /// * `deb_data`: The binary data of the `.deb` file.
    ///
    /// # Errors
    /// This function returns an error if there was an issue uploading the file.
    pub async fn upload_deb(
        &self,
        feed_name: &str,
        component_name: &str,
        deb_name: &str,
        deb_data: &[u8],
    ) -> Result<(), reqwest::Error> {
        deb::upload_deb(self, feed_name, component_name, deb_name, deb_data).await
    }
}
