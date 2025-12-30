use aw_client_rust::AwClient;

pub fn create_bucket(
    aw_client: &AwClient,
    bucket_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    aw_client.create_bucket(bucket_id, "network")?;
    Ok(())
}
