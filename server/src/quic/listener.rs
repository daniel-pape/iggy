use crate::binary::command;
use crate::quic::quic_sender::QuicSender;
use crate::server_error::ServerError;
use crate::streaming::clients::client_manager::Transport;
use crate::streaming::systems::system::System;
use crate::streaming::users::user_context::UserContext;
use iggy::bytes_serializable::BytesSerializable;
use iggy::command::Command;
use quinn::Endpoint;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::log::trace;
use tracing::{error, info};

const LISTENERS_COUNT: u32 = 10;
const INITIAL_BYTES_LENGTH: usize = 4;

pub fn start(endpoint: Endpoint, system: Arc<RwLock<System>>) {
    for _ in 0..LISTENERS_COUNT {
        let endpoint = endpoint.clone();
        let system = system.clone();
        tokio::spawn(async move {
            while let Some(incoming_connection) = endpoint.accept().await {
                info!(
                    "Incoming connection from client: {}",
                    incoming_connection.remote_address()
                );
                let system = system.clone();
                tokio::spawn(async move {
                    if let Err(error) = handle_connection(incoming_connection, system).await {
                        error!("Connection has failed: {}", error.to_string())
                    }
                });
            }
        });
    }
}

async fn handle_connection(
    incoming_connection: quinn::Connecting,
    system: Arc<RwLock<System>>,
) -> Result<(), ServerError> {
    let connection = incoming_connection.await?;
    let address = connection.remote_address();
    async {
        info!("Client has connected: {}", address);
        let client_id = system
            .read()
            .await
            .add_client(&address, Transport::Quic)
            .await;
        let mut user_context = UserContext::from_client_id(client_id);
        {
            let system = system.read().await;
            if !system.config.user.authentication_enabled {
                user_context.disable_authentication();
            }
        }

        loop {
            let stream = connection.accept_bi().await;
            let mut stream = match stream {
                Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                    info!("Connection closed");
                    system.read().await.delete_client(&address).await;
                    return Ok(());
                }
                Err(error) => {
                    error!("Error when handling QUIC stream: {:?}", error);
                    system.read().await.delete_client(&address).await;
                    return Err(error);
                }
                Ok(stream) => stream,
            };

            let request = stream.1.read_to_end(10 * 1024 * 1024).await;
            if request.is_err() {
                error!("Error when reading the QUIC request: {:?}", request);
                continue;
            }

            let request = request.unwrap();
            if request.len() < INITIAL_BYTES_LENGTH {
                error!(
                "Unable to read the QUIC request length, expected: {} bytes, received: {} bytes.",
                INITIAL_BYTES_LENGTH, request.len()
            );
                continue;
            }

            trace!("Trying to read command...");
            let length = &request[..INITIAL_BYTES_LENGTH];
            let length = u32::from_le_bytes(length.try_into().unwrap_or([0; 4]));
            let command = Command::from_bytes(&request[INITIAL_BYTES_LENGTH..]);
            if command.is_err() {
                error!(
                    "Error when reading the QUIC request command: {:?}",
                    command.err()
                );
                continue;
            }

            let command = command.unwrap();
            trace!(
                "Received a QUIC command: {}, payload size: {}",
                command,
                length
            );

            let result = command::handle(
                &command,
                &mut QuicSender {
                    send: stream.0,
                    recv: stream.1,
                },
                &mut user_context,
                system.clone(),
            )
            .await;
            if result.is_err() {
                error!("Error when handling the QUIC request: {:?}", result.err());
                continue;
            }
        }
    }
    .await?;
    Ok(())
}
