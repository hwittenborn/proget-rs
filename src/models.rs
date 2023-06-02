//! This module contains models of the data structures in the ProGet API.
use semver::Version;
use serde::Deserialize;

#[cfg(not(feature = "indexmap"))]
use std::collections::HashMap;
#[cfg(feature = "indexmap")]
use indexmap::IndexMap;

#[derive(Deserialize, Clone, PartialEq)]
pub enum Status {
    #[serde(rename = "OK")]
    Ok,
    Error,
}

#[derive(Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    pub application_name: String,
    pub database_status: Status,
    pub database_status_details: Option<String>,
    #[cfg(not(feature = "indexmap"))]
    pub extensions_installed: HashMap<String, Version>,
    #[cfg(feature = "indexmap")]
    pub extensions_installed: IndexMap<String, Version>,
    pub license_status: Status,
    pub license_status_detail: Option<String>,
    pub version_number: String,
    pub release_number: Version,
    pub service_status: Status,
    pub service_status_detail: Option<String>,
}
