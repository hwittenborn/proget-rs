pub mod deb;
use crate::{models, ClientData};

pub(crate) async fn health(client_data: &ClientData) -> crate::Result<models::Health> {
    let url = format!("{}health", client_data.server_url);
    let resp_text = client_data
        .http
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    Ok(serde_json::from_str(&resp_text)?)
}
