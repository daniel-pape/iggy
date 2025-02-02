use crate::streaming::users::permissioner::Permissioner;
use iggy::error::Error;

impl Permissioner {
    pub fn poll_messages(&self, user_id: u32, stream_id: u32, topic_id: u32) -> Result<(), Error> {
        if !self.enabled {
            return Ok(());
        }

        if self
            .users_that_can_poll_messages_from_all_streams
            .contains(&user_id)
        {
            return Ok(());
        }

        if self
            .users_that_can_poll_messages_from_specific_streams
            .contains(&(user_id, stream_id))
        {
            return Ok(());
        }

        let stream_permissions = self.users_streams_permissions.get(&(user_id, stream_id));
        if stream_permissions.is_none() {
            return Err(Error::Unauthorized);
        }

        let stream_permissions = stream_permissions.unwrap();
        if stream_permissions.poll_messages {
            return Ok(());
        }

        if stream_permissions.topics.is_none() {
            return Err(Error::Unauthorized);
        }

        let topic_permissions = stream_permissions.topics.as_ref().unwrap();
        if let Some(topic_permissions) = topic_permissions.get(&topic_id) {
            return match topic_permissions.poll_messages {
                true => Ok(()),
                false => Err(Error::Unauthorized),
            };
        }

        Err(Error::Unauthorized)
    }

    pub fn append_messages(
        &self,
        user_id: u32,
        stream_id: u32,
        topic_id: u32,
    ) -> Result<(), Error> {
        if !self.enabled {
            return Ok(());
        }

        if self
            .users_that_can_send_messages_to_all_streams
            .contains(&user_id)
        {
            return Ok(());
        }

        if self
            .users_that_can_send_messages_to_specific_streams
            .contains(&(user_id, stream_id))
        {
            return Ok(());
        }

        let stream_permissions = self.users_streams_permissions.get(&(user_id, stream_id));
        if stream_permissions.is_none() {
            return Err(Error::Unauthorized);
        }

        let stream_permissions = stream_permissions.unwrap();
        if stream_permissions.send_messages {
            return Ok(());
        }

        if stream_permissions.topics.is_none() {
            return Err(Error::Unauthorized);
        }

        let topic_permissions = stream_permissions.topics.as_ref().unwrap();
        if let Some(topic_permissions) = topic_permissions.get(&topic_id) {
            return match topic_permissions.send_messages {
                true => Ok(()),
                false => Err(Error::Unauthorized),
            };
        }

        Err(Error::Unauthorized)
    }
}
