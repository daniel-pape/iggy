use crate::cli::CliCommand;

use anyhow::{Context, Error, Result};
use async_trait::async_trait;
use iggy::client::Client;
use iggy::streams::create_stream::CreateStream;

#[derive(Debug)]
pub(crate) struct StreamCreate {
    id: u32,
    name: String,
}

impl StreamCreate {
    pub(crate) fn new(id: u32, name: String) -> Self {
        Self { id, name }
    }
}

#[async_trait]
impl CliCommand for StreamCreate {
    fn explain(&self) -> String {
        format!("create stream with id: {} and name: {}", self.id, self.name)
    }

    async fn execute_cmd(&mut self, client: &dyn Client) -> Result<(), Error> {
        client
            .create_stream(&CreateStream {
                stream_id: self.id,
                name: self.name.clone(),
            })
            .await
            .with_context(|| {
                format!(
                    "Problem creating stream (id: {} and name: {})",
                    self.id, self.name
                )
            })?;

        println!(
            "Stream with id: {} and name: {} created",
            self.id, self.name
        );

        Ok(())
    }
}
