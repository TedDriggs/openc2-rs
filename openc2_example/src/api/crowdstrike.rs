//! Non-functional CrowdStrike API client for demonstration purposes.
//! In a real implementation, this would be a separate crate providing the API client.

use std::{fmt, str::FromStr};

use openc2::{Error, ErrorAt};
use reqwest::{
    Url,
    header::{self, HeaderName, HeaderValue},
};

use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Debug, Clone, thiserror::Error)]
#[error("aid parse error")]
pub struct ParseAidError;

/// An AID (Agent ID) identifies a device in CrowdStrike.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, SerializeDisplay, DeserializeFromStr,
)]
pub struct Aid(String);

impl fmt::Display for Aid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<V> TryFrom<openc2::Target<V>> for Aid {
    type Error = Error;

    fn try_from(value: openc2::Target<V>) -> Result<Self, Self::Error> {
        let openc2::Target::Device(device) = value else {
            return Err(Error::validation("target is not a device"));
        };

        device.try_into().at("device")
    }
}

impl TryFrom<openc2::target::Device> for Aid {
    type Error = Error;

    fn try_from(value: openc2::target::Device) -> Result<Self, Self::Error> {
        value
            .device_id
            .as_ref()
            .ok_or_else(|| Error::validation("device_id is required"))
            .and_then(|s| {
                Aid::from_str(s).map_err(|e| {
                    Error::validation(format!("invalid device_id: {}", e)).at("device_id")
                })
            })
    }
}

impl FromStr for Aid {
    type Err = ParseAidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() || s.len() != 32 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ParseAidError);
        }
        Ok(Self(s.to_string()))
    }
}

impl From<&Aid> for String {
    fn from(value: &Aid) -> Self {
        value.0.clone()
    }
}

impl From<Aid> for String {
    fn from(value: Aid) -> Self {
        value.0
    }
}

/// Non-functional CrowdStrike API client for demonstration purposes.
pub struct Client {
    client: reqwest::Client,
    cloud: String,
}

impl Client {
    fn auth_header(
        client_secret: &str,
    ) -> Result<(HeaderName, HeaderValue), reqwest::header::InvalidHeaderValue> {
        let mut value = HeaderValue::from_str(&format!("Bearer {}", client_secret))?;
        value.set_sensitive(true);
        Ok((header::AUTHORIZATION, value))
    }

    pub fn new(cloud: impl Into<String>, client_secret: &str) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("openc2-consumer/0.1.0")
            .default_headers(
                [Client::auth_header(client_secret).unwrap()]
                    .into_iter()
                    .collect(),
            )
            .build()
            .unwrap();

        Self {
            client,
            cloud: cloud.into(),
        }
    }

    pub fn post(&self, url: impl AsRef<str>) -> reqwest::RequestBuilder {
        self.client.post(format!(
            "https://{}/{}",
            self.cloud,
            url.as_ref().trim_start_matches('/')
        ))
    }
}

impl Client {
    #[allow(dead_code)]
    pub async fn get_device_group_members(
        &self,
        group_id: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .get(format!("https://{}/devices/queries/devices/v1", self.cloud))
            .query(&[("filter", format!("group_ids:'{}'", group_id))])
            .send()
            .await?
            .error_for_status()
    }

    pub async fn contain_device(&self, id: &Aid) -> Result<reqwest::Response, reqwest::Error> {
        self.post("/devices/entities/devices-actions/v2")
            .json(&serde_json::json!({
                "action_name": "contain",
                "ids": [id],
                "parameters": [{"name": "containment_type", "value": "full"}]
            }))
            .send()
            .await?
            .error_for_status()
    }

    pub async fn detonate_url(&self, _url: &Url) -> Result<reqwest::Response, reqwest::Error> {
        todo!()
    }

    pub async fn delete_file(
        &self,
        _path: String,
        _device_id: Aid,
    ) -> Result<reqwest::Response, Error> {
        Err(Error::not_implemented(
            "CrowdStrike file deletion not implemented",
        ))
    }
}
