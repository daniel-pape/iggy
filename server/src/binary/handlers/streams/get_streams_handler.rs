use crate::binary::mapper;
use crate::binary::sender::Sender;
use crate::streaming::systems::system::System;
use crate::streaming::users::user_context::UserContext;
use anyhow::Result;
use iggy::error::Error;
use iggy::streams::get_streams::GetStreams;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::trace;

pub async fn handle(
    command: &GetStreams,
    sender: &mut dyn Sender,
    user_context: &UserContext,
    system: Arc<RwLock<System>>,
) -> Result<(), Error> {
    trace!("{command}");
    if !user_context.is_authenticated() {
        return Err(Error::Unauthenticated);
    }

    let system = system.read().await;
    system.permissioner.get_streams(user_context.user_id)?;
    let streams = system.get_streams();
    let streams = mapper::map_streams(&streams).await;
    sender.send_ok_response(streams.as_slice()).await?;
    Ok(())
}
