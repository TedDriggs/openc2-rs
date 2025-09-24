use std::{str::FromStr, sync::Arc};

use futures::{
    FutureExt, StreamExt,
    stream::{self, BoxStream},
};
use openc2::{
    Action, Error, ErrorAt, Hashes, Message, Nsid, Payload, Profile, StatusCode, TargetType,
    json::{Command, Headers, Response, Target},
    target::{self, Device},
};
use openc2_er::DeviceContainment;
use reqwest::Url;

use openc2_consumer::{Consume, Registration, ToRegistration, util::stream_just};

use crate::api::crowdstrike::{Aid, Client};

pub struct EndpointResponse {
    client: Arc<Client>,
}

impl EndpointResponse {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    pub async fn contain_device(&self, id: &Aid, _reason: Option<&str>) -> Result<Response, Error> {
        self.client
            .contain_device(id)
            .await
            .map_err(Error::custom)?;
        Ok(Response::new(StatusCode::Ok))
    }

    #[allow(unused_variables)]
    pub async fn stop_process(&self, device: &Aid, pid: u32) -> Result<Response, Error> {
        todo!()
    }

    async fn consume_contain_device(
        &self,
        msg: Message<Headers, Command>,
    ) -> Result<Response, Error> {
        let Command { args, profile, .. } = &msg.body;

        let mut errors = Error::accumulator();

        errors.handle(args.period.require_empty());

        let device = match &msg.body.target {
            Target::Device(device) => device,
            _ => {
                return Err(Error::validation("target must be a device").at("target"));
            }
        };

        let aid = errors.handle(
            device
                .device_id
                .as_ref()
                .ok_or_else(|| {
                    Error::validation("device_id is required")
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

        let er_ext = args
            .extensions
            .require::<openc2_er::Args>(&Nsid::ER)
            .map_err(Error::validation)
            .at(Nsid::ER)?;

        match er_ext.device_containment {
            Some(DeviceContainment::NetworkIsolation) => {}
            None => {
                errors.push(
                    Error::validation("device_containment is required")
                        .at("device_containment")
                        .at(Nsid::ER),
                );
            }
            Some(_) => {
                errors.push(
                    Error::not_implemented("must be network_isolation")
                        .at("device_containment")
                        .at(Nsid::ER),
                );
            }
        }

        errors.finish()?;

        self.contain_device(
            aid.as_ref().expect("AID was parsed"),
            args.comment.as_deref(),
        )
        .await
    }

    async fn consume_stop_process(
        &self,
        msg: Message<Headers, Command>,
    ) -> Result<Response, Error> {
        let Command { args, profile, .. } = &msg.body;

        let mut errors = Error::accumulator();

        errors.handle(args.period.require_empty());

        let process = match &msg.body.target {
            Target::Process(process) => process,
            _ => {
                return Err(Error::validation("target must be a process").at("target"));
            }
        };

        if let Some(profile) = profile
            && profile != &Nsid::ER
        {
            errors.push(Error::not_implemented("profile should be er").at("profile"));
        }

        errors.finish()?;

        let er_ext = args
            .extensions
            .require::<openc2_er::Args>(&Nsid::ER)
            .map_err(Error::validation)
            .at(Nsid::ER)?;

        let downstream = er_ext.require_downstream_device()?;

        if downstream.devices.len() > 1 {
            return Err(Error::validation("only one device is supported")
                .at("devices")
                .at("downstream_device")
                .at(Nsid::ER));
        }

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

        if aids.is_empty() {
            return Err(Error::validation("at least one device is required")
                .at("devices")
                .at("downstream_device")
                .at(Nsid::ER));
        }

        let pid = process.pid.ok_or_else(|| {
            Error::validation("process pid is required")
                .at("pid")
                .at("process")
                .at("target")
        })?;

        self.stop_process(&aids[0], pid).await
    }

    /// Validates a "delete file" command, returning the file path and list of target devices.
    fn validate_delete_file(
        &self,
        msg: Message<Headers, Command>,
    ) -> Result<(String, Vec<Device>), Error> {
        let Command { args, profile, .. } = msg.body;

        let mut errors = Error::accumulator();

        errors.handle(args.period.require_empty());

        let file = match msg.body.target {
            Target::File(file) => file,
            _ => {
                panic!("target must be a file, was {}", msg.body.target.kind());
            }
        };

        let path = errors.handle(file.path.ok_or_else(|| {
            Error::validation("file path is required")
                .at("path")
                .at("file")
                .at("target")
        }));

        if let Some(profile) = profile
            && profile != Nsid::ER
        {
            errors.push(Error::not_implemented("profile should be er").at("profile"));
        }

        let er_ext = args
            .extensions
            .require::<openc2_er::Args>(&Nsid::ER)
            .map_err(Error::validation)
            .at(Nsid::ER)?;

        let downstream = errors.handle(er_ext.require_downstream_device());

        errors.finish_with((path.unwrap(), downstream.unwrap().clone().devices))
    }

    fn delete_file_from_device<'a>(
        &'a self,
        device: Device,
        file_path: String,
    ) -> BoxStream<'a, Result<Response, Error>> {
        let Some(device_id) = &device.device_id else {
            return stream_just(Err(
                Error::validation("device_id is required").at("target.device.device_id")
            ));
        };

        let aid: Aid = match device_id.parse() {
            Ok(aid) => aid,
            Err(e) => {
                return stream_just(Err(Error::validation(format!("invalid device_id: {}", e))
                    .at("target.device.device_id")));
            }
        };

        stream::iter([Ok(Response::new(StatusCode::Processing))])
            .chain(stream::once(
                self.client
                    .delete_file(file_path.clone(), aid)
                    .map(|res| res.map(|_| Response::new(StatusCode::Ok))),
            ))
            .boxed()
    }
}

/// Returns a registration that specifies the [`er`](Nsid::ER) profile
/// and registers all the supported actions.
impl ToRegistration for EndpointResponse {
    fn to_registration(&self) -> Registration {
        Registration::new().with_actions([
            (Nsid::ER, Action::Contain, TargetType::Device),
            (Nsid::ER, Action::Restart, TargetType::Device),
            (Nsid::ER, Action::Stop, TargetType::Process),
            (Nsid::ER, Action::Deny, TargetType::File),
            (Nsid::ER, Action::Delete, TargetType::File),
        ])
    }
}

impl Consume for EndpointResponse {
    fn consume<'a>(
        &'a self,
        msg: Message<Headers, Command>,
    ) -> BoxStream<'a, Result<Response, Error>> {
        match msg.body.as_action_target() {
            (Action::Contain, Target::Device(_)) => {
                stream::once(self.consume_contain_device(msg)).boxed()
            }
            (Action::Delete, Target::File(_)) => {
                let (path, devices) = match self.validate_delete_file(msg) {
                    Ok(v) => v,
                    Err(e) => return stream_just(Err(e)),
                };

                stream::select_all(
                    devices
                        .into_iter()
                        .map(move |device| self.delete_file_from_device(device, path.clone())),
                )
                .boxed()
            }
            (Action::Stop, Target::Process(_)) => {
                stream::once(self.consume_stop_process(msg)).boxed()
            }
            (action, target) => stream_just(Err(Error::validation(format!(
                "unsupported action-target pair: {action} - {}",
                target.kind()
            )))),
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

    async fn consume_msg(&self, msg: Message<Headers, Command>) -> Result<Response, Error> {
        use openc2::{Action::*, Target::*};
        match msg.body.as_action_target() {
            (Detonate, Uri(url)) => self.detonate_uri(url).await,
            (Detonate, Artifact(artifact)) => self.detonate_artifact(artifact).await,
            (Scan, Artifact(artifact)) => self.scan_artifact(artifact).await,
            (action, target) => Err(Error::not_implemented_pair(action, &target.into())),
        }
    }
}

impl Profile for Sandbox {
    fn ns() -> &'static Nsid {
        SANDBOX_REF
    }
}

impl ToRegistration for Sandbox {
    fn to_registration(&self) -> Registration {
        Registration::new().with_actions([
            (SANDBOX, Action::Detonate, TargetType::Uri),
            (SANDBOX, Action::Detonate, TargetType::Artifact),
            (SANDBOX, Action::Scan, TargetType::Artifact),
        ])
    }
}

impl Consume for Sandbox {
    fn consume<'a>(
        &'a self,
        msg: Message<Headers, Command>,
    ) -> BoxStream<'a, Result<Response, Error>> {
        stream::once(self.consume_msg(msg)).boxed()
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
