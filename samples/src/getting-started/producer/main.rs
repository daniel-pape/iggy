use iggy::client::{Client, StreamClient, TopicClient};
use iggy::clients::client::IggyClient;
use iggy::identifier::Identifier;
use iggy::messages::send_messages::{Message, Partitioning, SendMessages};
use iggy::streams::create_stream::CreateStream;
use iggy::topics::create_topic::CreateTopic;
use std::error::Error;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

const STREAM_ID: u32 = 1;
const TOPIC_ID: u32 = 1;
const PARTITION_ID: u32 = 1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let mut client = IggyClient::default();
    client.connect().await?;
    init_system(&client).await;
    produce_messages(&client).await
}

async fn init_system(client: &IggyClient) {
    match client
        .create_stream(&CreateStream {
            stream_id: STREAM_ID,
            name: "sample-stream".to_string(),
        })
        .await
    {
        Ok(_) => info!("Stream was created."),
        Err(_) => warn!("Stream already exists and will not be created again."),
    }

    match client
        .create_topic(&CreateTopic {
            stream_id: Identifier::numeric(STREAM_ID).unwrap(),
            topic_id: TOPIC_ID,
            partitions_count: 1,
            name: "sample-topic".to_string(),
            message_expiry: None,
        })
        .await
    {
        Ok(_) => info!("Topic was created."),
        Err(_) => warn!("Topic already exists and will not be created again."),
    }
}

async fn produce_messages(client: &dyn Client) -> Result<(), Box<dyn Error>> {
    let interval = Duration::from_millis(500);
    info!(
        "Messages will be sent to stream: {}, topic: {}, partition: {} with interval {} ms.",
        STREAM_ID,
        TOPIC_ID,
        PARTITION_ID,
        interval.as_millis()
    );

    let mut current_id = 0;
    let messages_per_batch = 10;
    loop {
        let mut messages = Vec::new();
        for _ in 0..messages_per_batch {
            current_id += 1;
            let payload = format!("message-{current_id}");
            let message = Message::from_str(&payload)?;
            messages.push(message);
        }
        client
            .send_messages(&mut SendMessages {
                stream_id: Identifier::numeric(STREAM_ID)?,
                topic_id: Identifier::numeric(TOPIC_ID)?,
                partitioning: Partitioning::partition_id(PARTITION_ID),
                messages,
            })
            .await?;
        info!("Sent {messages_per_batch} message(s).");
        sleep(interval).await;
    }
}
