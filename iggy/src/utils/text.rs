use crate::error::Error;
use base64::engine::general_purpose;
use base64::Engine;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RESOURCE_NAME_REGEX: Regex = Regex::new(r"^[\w\.\-\s]+$").unwrap();
}

pub fn to_lowercase_non_whitespace(value: &str) -> String {
    value
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(".")
}

pub fn is_resource_name_valid(value: &str) -> bool {
    RESOURCE_NAME_REGEX.is_match(value)
}

pub fn from_base64_as_bytes(value: &str) -> Result<Vec<u8>, Error> {
    let result = general_purpose::STANDARD.decode(value);
    if result.is_err() {
        return Err(Error::InvalidFormat);
    }

    Ok(result.unwrap())
}
