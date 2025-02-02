use crate::bytes_serializable::BytesSerializable;
use crate::command::CommandPayload;
use crate::consumer::{Consumer, ConsumerKind};
use crate::error::Error;
use crate::identifier::Identifier;
use crate::validatable::Validatable;
use bytes::BufMut;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GetConsumerOffset {
    #[serde(flatten)]
    pub consumer: Consumer,
    #[serde(skip)]
    pub stream_id: Identifier,
    #[serde(skip)]
    pub topic_id: Identifier,
    #[serde(default = "default_partition_id")]
    pub partition_id: Option<u32>,
}

impl Default for GetConsumerOffset {
    fn default() -> Self {
        GetConsumerOffset {
            consumer: Consumer::default(),
            stream_id: Identifier::default(),
            topic_id: Identifier::default(),
            partition_id: default_partition_id(),
        }
    }
}

impl CommandPayload for GetConsumerOffset {}

fn default_partition_id() -> Option<u32> {
    Some(1)
}

impl Validatable<Error> for GetConsumerOffset {
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl FromStr for GetConsumerOffset {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parts = input.split('|').collect::<Vec<&str>>();
        if parts.len() != 5 {
            return Err(Error::InvalidCommand);
        }

        let consumer_kind = ConsumerKind::from_str(parts[0])?;
        let consumer_id = parts[1].parse::<u32>()?;
        let consumer = Consumer {
            kind: consumer_kind,
            id: consumer_id,
        };
        let stream_id = parts[2].parse::<Identifier>()?;
        let topic_id = parts[3].parse::<Identifier>()?;
        let partition_id = parts[4].parse::<u32>()?;
        let command = GetConsumerOffset {
            consumer,
            stream_id,
            topic_id,
            partition_id: Some(partition_id),
        };
        command.validate()?;
        Ok(command)
    }
}

impl BytesSerializable for GetConsumerOffset {
    fn as_bytes(&self) -> Vec<u8> {
        let consumer_bytes = self.consumer.as_bytes();
        let stream_id_bytes = self.stream_id.as_bytes();
        let topic_id_bytes = self.topic_id.as_bytes();
        let mut bytes = Vec::with_capacity(
            4 + consumer_bytes.len() + stream_id_bytes.len() + topic_id_bytes.len(),
        );
        bytes.extend(consumer_bytes);
        bytes.extend(stream_id_bytes);
        bytes.extend(topic_id_bytes);
        if let Some(partition_id) = self.partition_id {
            bytes.put_u32_le(partition_id);
        } else {
            bytes.put_u32_le(0);
        }
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<GetConsumerOffset, Error> {
        if bytes.len() < 15 {
            return Err(Error::InvalidCommand);
        }

        let mut position = 0;
        let consumer_kind = ConsumerKind::from_code(bytes[0])?;
        let consumer_id = u32::from_le_bytes(bytes[1..5].try_into()?);
        let consumer = Consumer {
            kind: consumer_kind,
            id: consumer_id,
        };
        position += 5;
        let stream_id = Identifier::from_bytes(&bytes[position..])?;
        position += stream_id.get_size_bytes() as usize;
        let topic_id = Identifier::from_bytes(&bytes[position..])?;
        position += topic_id.get_size_bytes() as usize;
        let partition_id = u32::from_le_bytes(bytes[position..position + 4].try_into()?);
        let partition_id = if partition_id == 0 {
            None
        } else {
            Some(partition_id)
        };
        let command = GetConsumerOffset {
            consumer,
            stream_id,
            topic_id,
            partition_id,
        };
        command.validate()?;
        Ok(command)
    }
}

impl Display for GetConsumerOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}",
            self.consumer,
            self.stream_id,
            self.topic_id,
            self.partition_id.unwrap_or(0)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_serialized_as_bytes() {
        let command = GetConsumerOffset {
            consumer: Consumer::new(1),
            stream_id: Identifier::numeric(2).unwrap(),
            topic_id: Identifier::numeric(3).unwrap(),
            partition_id: Some(4),
        };

        let bytes = command.as_bytes();
        let mut position = 0;
        let consumer_kind = ConsumerKind::from_code(bytes[0]).unwrap();
        let consumer_id = u32::from_le_bytes(bytes[1..5].try_into().unwrap());
        let consumer = Consumer {
            kind: consumer_kind,
            id: consumer_id,
        };
        position += 5;
        let stream_id = Identifier::from_bytes(&bytes[position..]).unwrap();
        position += stream_id.get_size_bytes() as usize;
        let topic_id = Identifier::from_bytes(&bytes[position..]).unwrap();
        position += topic_id.get_size_bytes() as usize;
        let partition_id = u32::from_le_bytes(bytes[position..position + 4].try_into().unwrap());

        assert!(!bytes.is_empty());
        assert_eq!(consumer, command.consumer);
        assert_eq!(stream_id, command.stream_id);
        assert_eq!(topic_id, command.topic_id);
        assert_eq!(Some(partition_id), command.partition_id);
    }

    #[test]
    fn should_be_deserialized_from_bytes() {
        let consumer = Consumer::new(1);
        let stream_id = Identifier::numeric(2).unwrap();
        let topic_id = Identifier::numeric(3).unwrap();
        let partition_id = 4u32;

        let consumer_bytes = consumer.as_bytes();
        let stream_id_bytes = stream_id.as_bytes();
        let topic_id_bytes = topic_id.as_bytes();
        let mut bytes = Vec::with_capacity(
            4 + consumer_bytes.len() + stream_id_bytes.len() + topic_id_bytes.len(),
        );
        bytes.extend(consumer_bytes);
        bytes.extend(stream_id_bytes);
        bytes.extend(topic_id_bytes);
        bytes.put_u32_le(partition_id);

        let command = GetConsumerOffset::from_bytes(&bytes);
        assert!(command.is_ok());

        let command = command.unwrap();
        assert_eq!(consumer, command.consumer);
        assert_eq!(command.stream_id, stream_id);
        assert_eq!(command.topic_id, topic_id);
        assert_eq!(command.partition_id, Some(partition_id));
    }

    #[test]
    fn should_be_read_from_string() {
        let consumer = Consumer::new(1);
        let stream_id = Identifier::numeric(2).unwrap();
        let topic_id = Identifier::numeric(3).unwrap();
        let partition_id = 4u32;
        let input = format!("{consumer}|{stream_id}|{topic_id}|{partition_id}");
        let command = GetConsumerOffset::from_str(&input);
        assert!(command.is_ok());

        let command = command.unwrap();
        assert_eq!(command.consumer, consumer);
        assert_eq!(command.stream_id, stream_id);
        assert_eq!(command.topic_id, topic_id);
        assert_eq!(command.partition_id, Some(partition_id));
    }
}
