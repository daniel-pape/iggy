use anyhow::Result;
use clap::Parser;
use iggy::client_provider;
use iggy::client_provider::ClientProviderConfig;
use iggy::clients::client::{IggyClient, IggyClientConfig, PollMessagesConfig, StoreOffsetKind};
use iggy::consumer::{Consumer, ConsumerKind};
use iggy::identifier::Identifier;
use iggy::messages::poll_messages::{PollMessages, PollingStrategy};
use iggy::models::messages::Message;
use samples::shared::args::Args;
use samples::shared::messages::*;
use samples::shared::system;
use std::error::Error;
use std::sync::Arc;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    tracing_subscriber::fmt::init();
    info!(
        "Message envelope consumer has started, selected transport: {}",
        args.transport
    );
    let client_provider_config = Arc::new(ClientProviderConfig::from_args(args.to_sdk_args())?);
    let client = client_provider::get_raw_client(client_provider_config).await?;
    let client = IggyClient::builder(client)
        .with_config(IggyClientConfig {
            poll_messages: PollMessagesConfig {
                interval: args.interval,
                store_offset_kind: StoreOffsetKind::WhenMessagesAreProcessed,
            },
            ..Default::default()
        })
        .build();
    system::init_by_consumer(&args, &client).await;
    consume_messages(&args, &client).await
}

async fn consume_messages(args: &Args, client: &IggyClient) -> Result<(), Box<dyn Error>> {
    info!("Messages will be polled by consumer: {} from stream: {}, topic: {}, partition: {} with interval {} ms.",
        args.consumer_id, args.stream_id, args.topic_id, args.partition_id, args.interval);
    client
        .start_polling_messages(
            PollMessages {
                consumer: Consumer {
                    kind: ConsumerKind::from_code(args.consumer_kind)?,
                    id: args.consumer_id,
                },
                stream_id: Identifier::numeric(args.stream_id)?,
                topic_id: Identifier::numeric(args.topic_id)?,
                partition_id: Some(args.partition_id),
                strategy: PollingStrategy::next(),
                count: args.messages_per_batch,
                auto_commit: true,
            },
            Some(|message| {
                let result = handle_message(&message);
                if let Err(e) = result {
                    warn!("Error handling message: {}", e);
                }
            }),
            None,
        )
        .await?;
    Ok(())
}

fn handle_message(message: &Message) -> Result<(), Box<dyn Error>> {
    // The payload can be of any type as it is a raw byte array. In this case it's a JSON string.
    let json = std::str::from_utf8(&message.payload)?;
    // The message envelope can be used to send the different types of messages to the same topic.
    let envelope = serde_json::from_str::<Envelope>(json)?;
    info!(
        "Handling message type: {} at offset: {}...",
        envelope.message_type, message.offset
    );
    match envelope.message_type.as_str() {
        ORDER_CREATED_TYPE => {
            let order_created = serde_json::from_str::<OrderCreated>(&envelope.payload)?;
            info!("{:#?}", order_created);
        }
        ORDER_CONFIRMED_TYPE => {
            let order_confirmed = serde_json::from_str::<OrderConfirmed>(&envelope.payload)?;
            info!("{:#?}", order_confirmed);
        }
        ORDER_REJECTED_TYPE => {
            let order_rejected = serde_json::from_str::<OrderRejected>(&envelope.payload)?;
            info!("{:#?}", order_rejected);
        }
        _ => {
            warn!("Received unknown message type: {}", envelope.message_type);
        }
    }
    Ok(())
}
