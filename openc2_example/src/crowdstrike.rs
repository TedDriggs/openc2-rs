use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use futures::future::join_all;
use openc2::{
    Action, Error, ErrorAt, Hashes, Headers, Message, Nsid, Payload, Profile, StatusCode,
    TargetType,
    json::{Command, Response, Target},
    target,
};
use openc2_er::DeviceContainment;
use reqwest::Url;

use openc2_consumer::{Consume, Registration};

use crate::api::crowdstrike::{Aid, Client};

pub struct EndpointResponse {
    client: Arc<Client>,
}

impl EndpointResponse {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    pub async fn contain_device(&self, id: &Aid) -> Result<Response, Error> {
        self.client
            .contain_device(id)
            .await
            .map_err(Error::custom)?;
        Ok(Response::new(StatusCode::Ok))
    }
}

/// Returns a registration that specifies the [`er`](Nsid::ER) profile
/// and registers all the supported actions.
impl From<EndpointResponse> for Registration {
    fn from(actuator: EndpointResponse) -> Self {
        Registration::new(actuator).with_actions([(Nsid::ER, Action::Contain, TargetType::Device)])
    }
}

#[async_trait]
impl Consume for EndpointResponse {
    async fn consume(&self, msg: Message<Headers, Command>) -> Result<Response, Error> {
        let Command {
            action,
            target,
            args,
            profile,
            ..
        } = &msg.body;

        match (action, target) {
            (Action::Contain, Target::Device(device)) => {
                let mut errors = Error::accumulator();

                errors.handle(args.period.require_empty());

                let aid = errors.handle(
                    device
                        .device_id
                        .as_ref()
                        .ok_or_else(|| {
                            Error::validation("device_id is required")
                                .at("device_id")
                                .at("device")
                                .at("target")
                        })
                        .and_then(|s| {
                            Aid::from_str(s).map_err(|e| {
                                Error::validation(format!("invalid device_id: {}", e))
                                    .at("target.device.device_id")
                            })
                        }),
                );

                if let Some(profile) = profile
                    && profile != &Nsid::ER
                {
                    errors.push(Error::not_implemented("profile should be er").at("profile"));
                }

                if let Some(er_args) = errors.handle(
                    args.extensions
                        .require::<openc2_er::Args>(&Nsid::ER)
                        .map_err(Error::validation)
                        .at(Nsid::ER),
                ) && er_args.device_containment != Some(DeviceContainment::NetworkIsolation)
                {
                    errors.push(
                        Error::not_implemented("must be network_isolation")
                            .at("device_containment")
                            .at(Nsid::ER),
                    );
                }

                errors.finish()?;

                self.contain_device(aid.as_ref().expect("AID was parsed"))
                    .await
            }
            (Action::Delete, Target::File(file)) => {
                let Some(path) = &file.path else {
                    return Err(Error::validation("file path is required")
                        .at("path")
                        .at("file")
                        .at("target"));
                };

                let er_ext = args
                    .extensions
                    .require::<openc2_er::Args>(&Nsid::ER)
                    .map_err(Error::validation)
                    .at(Nsid::ER)?;

                let Some(downstream) = &er_ext.downstream_device else {
                    return Err(Error::validation("downstream_device is required")
                        .at("downstream_device")
                        .at(Nsid::ER));
                };

                let aids = downstream
                    .devices
                    .iter()
                    .filter_map(|s| s.device_id.as_deref())
                    .map(Aid::from_str)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| {
                        Error::validation(format!("invalid device id: {}", e))
                            .at("devices")
                            .at("downstream_device")
                            .at(Nsid::ER)
                    })?;

                let results =
                    join_all(aids.iter().map(|aid| self.client.delete_file(path, aid))).await;

                results
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::custom)?;

                Ok(Response::new(StatusCode::Ok))
            }
            (action, target) => Err(Error::validation(format!(
                "unsupported action-target pair: {action} - {}",
                target.kind()
            ))),
        }
    }
}

const SANDBOX: Nsid = Nsid::new_const("sandbox");
const SANDBOX_REF: &Nsid = &SANDBOX;

pub struct Sandbox {
    client: Arc<Client>,
}

impl Sandbox {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    #[allow(unused_variables)]
    pub async fn detonate_artifact(&self, artifact: &target::Artifact) -> Result<Response, Error> {
        if let Some(hashes) = &artifact.hashes {
            let sha256 = require_sha256(hashes).at("artifact").at("target")?;
            todo!()
        }

        if let Some(payload) = &artifact.payload {
            match payload {
                Payload::Url(url) => todo!(),
                Payload::Binary(bytes) => todo!(),
            }
        }

        Err(missing_sha256_and_payload())
    }

    pub async fn detonate_uri(&self, url: &Url) -> Result<Response, Error> {
        self.client.detonate_url(url).await.map_err(Error::custom)?;
        Ok(Response::new(StatusCode::Ok))
    }

    #[allow(unused_variables)]
    pub async fn scan_artifact(&self, artifact: &target::Artifact) -> Result<Response, Error> {
        if let Some(hashes) = &artifact.hashes {
            let sha256 = require_sha256(hashes).at("artifact").at("target")?;
            todo!()
        }

        if let Some(payload) = &artifact.payload {
            match payload {
                Payload::Url(url) => todo!(),
                Payload::Binary(bytes) => todo!(),
            }
        }

        Err(missing_sha256_and_payload())
    }
}

impl Profile for Sandbox {
    fn ns() -> &'static Nsid {
        SANDBOX_REF
    }
}

impl From<Sandbox> for Registration {
    fn from(actuator: Sandbox) -> Self {
        Registration::new(actuator).with_actions([
            (SANDBOX, Action::Detonate, TargetType::Uri),
            (SANDBOX, Action::Detonate, TargetType::File),
        ])
    }
}

#[async_trait]
impl Consume for Sandbox {
    async fn consume(&self, msg: Message<Headers, Command>) -> Result<Response, Error> {
        use openc2::{Action::*, Target::*};
        match msg.body.as_action_target() {
            (Detonate, Uri(url)) => self.detonate_uri(url).await,
            (Detonate, Artifact(artifact)) => self.detonate_artifact(artifact).await,
            (Scan, Artifact(artifact)) => self.scan_artifact(artifact).await,
            (action, target) => Err(Error::not_implemented_pair(action, &target.into())),
        }
    }
}

fn require_sha256(hashes: &Hashes) -> Result<&str, Error> {
    hashes
        .sha256
        .as_deref()
        .ok_or_else(|| Error::validation("sha256 hash is required").at("hashes"))
}

fn missing_sha256_and_payload() -> Error {
    Error::validation("artifact must have either sha256 or bytes")
        .at("artifact")
        .at("target")
}
