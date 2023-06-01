use crate::ClientData;

pub(crate) async fn upload_deb(
    client_data: &ClientData,
    feed_name: &str,
    component_name: &str,
    deb_name: &str,
    deb_data: &[u8],
) -> crate::Result<()> {
    let url = format!(
        "{}debian-packages/upload/{}/{}/{}",
        client_data.server_url, feed_name, component_name, deb_name
    );

    client_data
        .http
        .post(url)
        .body(deb_data.to_vec())
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
