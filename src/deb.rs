use crate::Client;

pub(crate) async fn upload_deb(
    client: &Client,
    feed_name: &str,
    component_name: &str,
    deb_name: &str,
    deb_data: &[u8],
) -> Result<(), reqwest::Error> {
    let url = format!(
        "{}/debian-packages/upload/{}/{}/{}",
        client.server_url, feed_name, component_name, deb_name
    );

    client
        .http
        .post(url)
        .body(deb_data.to_vec())
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
