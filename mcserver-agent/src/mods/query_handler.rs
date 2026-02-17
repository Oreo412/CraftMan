use protocol::query_options::QueryOptions;
use rust_mc_status::JavaStatus;
use rust_mc_status::McClient;
use rust_mc_status::ServerData;
use rust_mc_status::error::McError;

pub struct QueryHandler {
    client: McClient,
    message_id: u32,
    port: u32,
    options: QueryOptions,
}

impl QueryHandler {
    pub fn new(port: u32, message_id: u32, options: QueryOptions) -> Self {
        QueryHandler {
            client: McClient::new(),
            message_id,
            port,
            options,
        }
    }

    pub async fn ping(&self) -> Result<JavaStatus, McError> {
        if let ServerData::Java(javastatus) = self
            .client
            .ping_java(&format!("localhost:{}", self.port))
            .await?
            .data
        {
            Ok(javastatus)
        } else {
            Err(McError::InvalidResponse(
                "The returned data in the ping function is not a java status. I don't think this is supposed to be possible".to_string()
            ))
        }
    }
}
