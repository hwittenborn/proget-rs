//! An unofficial Rust SDK for [Proget](https://inedo.com/proget).
//!
//! Most use cases will involve beginning with a [`Client`]. Please start there if you're trying to find your way around the library.
pub use reqwest;
use std::fmt;
pub use url;
use url::Url;

/// Errors that can happen in [`ClientBuilder`].
#[derive(Debug)]
pub enum ClientBuilderError {
    MissingServerUrl,
    MissingServerToken,
    InvalidServerUrl(url::ParseError),
}

impl fmt::Display for ClientBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            Self::MissingServerUrl => "missing server url".to_string(),
            Self::MissingServerToken => "missing server token".to_string(),
            Self::InvalidServerUrl(err) => err.to_string(),
        };
        write!(f, "{}", err)
    }
}

/// Builder to create and manage various aspects of a [`Client`].
pub struct ClientBuilder {
    server_url: Option<String>,
    api_token: Option<String>,
}

impl ClientBuilder {
    /// Create a new builder object.
    fn new() -> Self {
        Self {
            server_url: None,
            api_token: None,
        }
    }

    /// Set the server URL.
    pub fn server<T: ToString>(mut self, url: T) -> Self {
        self.server_url = Some(url.to_string());
        self
    }

    /// Set the API token used in requests.
    pub fn token<T: ToString>(mut self, token: T) -> Self {
        self.api_token = Some(token.to_string());
        self
    }

    /// Create a [`Client`] with the options set on this builder.
    pub fn build(self) -> Result<Client, ClientBuilderError> {
        if self.server_url.is_none() {
            Err(ClientBuilderError::MissingServerUrl)
        } else if self.api_token.is_none() {
            Err(ClientBuilderError::MissingServerToken)
        } else if let Err(err) = Url::parse(self.server_url.as_ref().unwrap()) {
            Err(ClientBuilderError::InvalidServerUrl(err))
        } else {
            Ok(Client::new(
                self.server_url.unwrap(),
                self.api_token.unwrap(),
            ))
        }
    }
}

/// A struct representing a user of a ProGet instance.
pub struct Client {
    http: reqwest::Client,
    server_url: String,
}

impl Client {
    /// Create a new client.
    fn new<T: ToString>(server_url: T, api_token: T) -> Self {
        let http_client = reqwest::Client::builder()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION"),
            ))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert("X-ApiKey", api_token.to_string().parse().unwrap());
                headers
            })
            .build()
            .unwrap();

        Self {
            http: http_client,
            server_url: server_url.to_string(),
        }
    }

    /// Create a new [`ClientBuilder`] to configure a client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Upload a `.deb` package.
    ///
    /// # Arguments
    /// * `feed_name`: The [feed](https://docs.inedo.com/docs/proget-feeds-feed-overview) to upload the `.deb` package to.
    /// * `component_name`: The component in the APT repository to upload the deb to. For example, this would be `component` in `deb https://proget.example.com deb-packages component`.
    /// * `deb_name`: The name of the `.deb` file to register the package under (i.e. `pkg_1.0.0-1_amd64.deb`).
    /// * `deb_data`: The binary data of the `.deb` file.
    pub async fn upload_deb<T: ToString>(
        &self,
        feed_name: T,
        component_name: T,
        deb_name: T,
        deb_data: &[u8],
    ) -> Result<(), reqwest::Error> {
        let url = format!(
            "{}/debian-packages/upload/{}/{}/{}",
            self.server_url,
            feed_name.to_string(),
            component_name.to_string(),
            deb_name.to_string()
        );
        self.http
            .post(url)
            .body(deb_data.to_vec())
            .send()
            .await
            .map(|_| ())
    }
}
