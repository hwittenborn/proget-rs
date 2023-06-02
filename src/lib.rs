//! A library providing a client for the [ProGet](https://inedo.com/proget) API.
//!
//! This library is **heavily** a work in progress, and stability is currently **not guaranteed**.
//! The library also needs a plethora of features to be added still - if there's something you'd
//! like added that's missing, feel free to make an issue or send in a PR on the
//! [GitHub repository](https://github.com/hwittenborn/proget-rust-sdk).
//!
//! Most use cases will involve beginning with a [`Client`]. Please start there
//! if you're trying to find your way around the library.
//!
//! # Feature flags
//! - `rustls-tls`: Use `rustls` as the TLS backend. Uses the system's native backend when not
//!   enabled.
//! - `indexmap`: Use [`IndexMap`] instead of [`HashMap`] for items in
//!   [`models`].
mod api;
pub mod models;

pub use reqwest;
pub use semver;

#[cfg(feature = "__docs")]
use indexmap::IndexMap;
#[cfg(feature = "__docs")]
use std::collections::HashMap;

use reqwest::{header::HeaderMap, Url};
use std::{marker::PhantomData, ops::Deref};
use thiserror::Error as ThisError;

/// The user agent we use in requests.
static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Alias for a [`std::result::Result`] with the error type always being [`crate::Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// The errors that may occur throughout this crate.
#[derive(ThisError, Debug)]
pub enum Error {
    /// An error while making an HTTP request.
    #[error("{0}")]
    Http(#[from] reqwest::Error),
    /// An error while parsing JSON data.
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

// Traits to differentiate between authenticated and anonymous clients.
mod private {
    pub trait AuthType {}
}

/// The type needed for an authenticated [`Client`].
#[derive(Clone)]
pub struct Anon;

/// The type needed for an anonymous [`Client`].
#[derive(Clone)]
pub struct Auth;

impl private::AuthType for Anon {}
impl private::AuthType for Auth {}

/// The client data for [`Client`].
#[derive(Clone)]
struct ClientData {
    http: reqwest::Client,
    server_url: Url,
}

/// A struct representing a user of a ProGet instance.
///
/// Most methods require authentication in order to run. For the methods that don't, you can call
/// [`Client::new_anon`] to make a new client without any authentication. If you'd like to run any
/// authenticated calls, use [`Client::new_auth`] instead.
///
/// All methods on the [`Anon`] version of the client are automatically available on the [`Auth`]
/// version, so there's no need to make two separate clients.
//
// This code is a bit messy - it's used this way so the documentation looks cleaner. While usually
// you do want the code to be what looks code, having it this way makes the documentation look
// really good, and it's not something I want to sacrafice right now.
//
// # How it works:
// - When the client is a `Client<Anon>`, `client_data` is the `Some` variant, and `anon_client` is
//   `None` (since we already have a `Client<Anon>`.
// - When the client is a `Client<Auth>`, `client_data` is `None`, and `anon_client` is `Some`,
//   pointing to the `Client<Anon>`.
//
// These generics on the `Client` don't really mean anything, they both ultimately point to
// `ClientData`, which is what contains all the data. The `new_anon` and `new_auth` functions below
// are what set the data in `ClientData`, with `new_auth` just setting some authentication data. We
// just have the two separate types for extra type safety - if you call `Client::new_anon`, you
// won't be able to call any functions from `Client<Auth>` - the error messages from Rust are also
// amazing, and hint really good that an authenticated client from `Client::new_auth` should be
// used instead.
//
// If you can find a cleaner way to implement this be my guest, PRs welcomed! :D
#[derive(Clone)]
pub struct Client<A: private::AuthType> {
    client_data: Option<ClientData>,
    anon_client: Option<Box<Client<Anon>>>,
    _phantom: PhantomData<A>,
}

/// Functions to create and interact with ProGet without authentication.
impl Client<Anon> {
    /// Get the client data.
    fn client_data(&self) -> &ClientData {
        self.client_data.as_ref().unwrap()
    }

    /// Create a new anonymous, unauthenticated client.
    pub fn new_anon(server_url: Url) -> Self {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap();
        let client_data = ClientData { http, server_url };

        Self {
            client_data: Some(client_data),
            anon_client: None,
            _phantom: PhantomData,
        }
    }

    /// Get health/status information.
    pub async fn health(&self) -> crate::Result<models::Health> {
        api::health(self.client_data()).await
    }
}

/// Functions to create and interact with ProGet with authentication.
impl Client<Auth> {
    /// Create a new authenticated client.
    pub fn new_auth(server_url: Url, api_token: &str) -> Self {
        let mut headers = HeaderMap::new();
        let auth_key = base64::encode(format!("api:{api_token}"));
        let auth_header = format!("Basic {auth_key}");
        headers.insert("Authorization", auth_header.parse().unwrap());

        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .build()
            .unwrap();
        let client_data = ClientData { http, server_url };
        let client = Client {
            client_data: Some(client_data),
            anon_client: None,
            _phantom: PhantomData,
        };

        Self {
            client_data: None,
            anon_client: Some(Box::new(client)),
            _phantom: PhantomData,
        }
    }

    /// Upload a `.deb` package.
    ///
    /// # Arguments
    /// * `feed_name`: The [feed](https://docs.inedo.com/docs/proget-feeds-feed-overview) to upload the `.deb` package to.
    /// * `component_name`: The component in the APT repository to upload the deb to. For example, this would be `bionic` in `deb https://proget.inedo.com deb-packages bionic`.
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
    ) -> crate::Result<()> {
        api::deb::upload_deb(
            self.client_data(),
            feed_name,
            component_name,
            deb_name,
            deb_data,
        )
        .await
    }
}

/// Automatic conversion of an [`Auth`] client into an [`Anon`] client, which allows
/// anonymous-access functions like [`Client::health`] to be accessed from the authenticated
/// client.
impl Deref for Client<Auth> {
    type Target = Client<Anon>;

    fn deref(&self) -> &Self::Target {
        self.anon_client.as_ref().unwrap()
    }
}
