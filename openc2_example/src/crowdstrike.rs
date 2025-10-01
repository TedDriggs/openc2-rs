use std::sync::Arc;

use futures::{
    FutureExt, StreamExt,
    stream::{self, BoxStream},
};
use openc2::{
    Action, Error, ErrorAt, Hashes, Message, Nsid, Payload, Profile, StatusCode, TargetType,
    json::{Command, Headers, Response, Target},
    target,
};
use openc2_er::DeviceContainment;
use reqwest::Url;

use openc2_consumer::{Consume, Registration, ToRegistration, util::stream_just};
use staging::Staging;

use crate::api::crowdstrike::{Aid, Client};

fn aids_from_downstream_devices(
    downstream: &openc2_er::DownstreamDevice,
) -> Result<Vec<Aid>, Error> {
    let mut errors = Error::accumulator();

    if !downstream.device_groups.is_empty() {
        errors.push(Error::not_implemented("device_groups are not supported").at("device_groups"));
    }

    if downstream.tenant_id.is_some() {
        errors.push(Error::not_implemented("tenant_id is not supported").at("tenant_id"));
    }

    if downstream.devices.is_empty() {
        errors.push(Error::validation("at least one device is required").at("devices"));
    }

    let aids = downstream
        .devices
        .iter()
        .enumerate()
        .map(|(idx, device)| {
            device
                .try_into()
                .at(idx)
                .at("devices")
                .at("downstream_device")
        })
        .filter_map(|res| errors.handle(res))
        .collect::<Vec<Aid>>();

    errors.finish_with(aids)
}

#[derive(Debug, Clone, Staging)]
#[staging(error = Error, additional_errors)]
pub struct ContainDeviceArgs {
    pub device_id: Aid,
    #[allow(
        dead_code,
        reason = "this should be passed to the API but is not yet implemented"
    )]
    pub reason: Option<String>,
}

impl TryFrom<Message<Headers, Command>> for ContainDeviceArgs {
    type Error = Error;

    fn try_from(msg: Message<Headers, Command>) -> Result<Self, Self::Error> {
        let Command {
            args,
            target,
            profile,
            ..
        } = msg.body;

        let mut result = ContainDeviceArgsStaging {
            device_id: target.try_into(),
            reason: Ok(args.comment),
            additional_errors: vec![],
        };

        let er_ext = result.handle(
            args.extensions
                .require::<openc2_er::Args>(&Nsid::ER)
                .map_err(Error::validation)
                .at(Nsid::ER),
        );

        if let Some(ext) = er_ext {
            match ext.device_containment {
                Some(DeviceContainment::NetworkIsolation) => {}
                None => {
                    result.additional_errors.push(
                        Error::validation("device_containment is required")
                            .at("device_containment")
                            .at(Nsid::ER),
                    );
                }
                Some(_) => {
                    result.additional_errors.push(
                        Error::not_implemented("must be network_isolation")
                            .at("device_containment")
                            .at(Nsid::ER),
                    );
                }
            }
        }

        if let Some(profile) = profile
            && profile != Nsid::ER
        {
            result
                .additional_errors
                .push(Error::validation("profile should be er").at("profile"));
        }

        result.try_into()
    }
}

#[derive(Debug, Clone, Staging)]
#[staging(error = Error, additional_errors)]
#[allow(dead_code, reason = "fields will be used in future implementation")]
pub struct StopProcessArgs {
    pub device_id: Aid,
    pub pid: u32,
}

impl TryFrom<Message<Headers, Command>> for StopProcessArgs {
    type Error = Error;

    fn try_from(msg: Message<Headers, Command>) -> Result<Self, Self::Error> {
        let Command { args, target, .. } = msg.body;

        let er_ext = args
            .extensions
            .require::<openc2_er::Args>(&Nsid::ER)
            .map_err(Error::validation)
            .at(Nsid::ER);

        let result = StopProcessArgsStaging {
            pid: match target {
                Target::Process(process) => process.pid.ok_or_else(|| {
                    Error::validation("process pid is required")
                        .at("pid")
                        .at("process")
                        .at("target")
                }),
                _ => Err(Error::validation("target must be a process").at("target")),
            },
            device_id: er_ext
                .as_ref()
                .map_err(Clone::clone)
                .and_then(|args| args.require_downstream_device())
                .and_then(|downstream| {
                    if downstream.devices.len() != 1 {
                        return Err(Error::validation("exactly one device is required")
                            .at("devices")
                            .at("downstream_device"));
                    }
                    let device = &downstream.devices[0];
                    device
                        .clone()
                        .try_into()
                        .at(0)
                        .at("devices")
                        .at("downstream_device")
                })
                .at(Nsid::ER),
            additional_errors: vec![],
        };

        result.try_into()
    }
}

#[derive(Debug, Clone, Staging)]
#[staging(error = Error, additional_errors)]
pub struct DeleteFileArgs {
    pub file_path: String,
    pub target_devices: Vec<Aid>,
}

impl TryFrom<Message<Headers, Command>> for DeleteFileArgs {
    type Error = Error;

    fn try_from(msg: Message<Headers, Command>) -> Result<Self, Self::Error> {
        let mut result = DeleteFileArgsStaging {
            file_path: match msg.body.target {
                Target::File(file) => file.path.ok_or_else(|| {
                    Error::validation("file path is required")
                        .at("path")
                        .at("file")
                        .at("target")
                }),
                _ => Err(Error::validation("target must be a file").at("target")),
            },
            target_devices: ({
                let er_ext = msg
                    .body
                    .args
                    .extensions
                    .require::<openc2_er::Args>(&Nsid::ER)
                    .map_err(Error::validation);

                er_ext
                    .as_ref()
                    .map_err(Clone::clone)
                    .and_then(|ext| ext.require_downstream_device())
                    .and_then(aids_from_downstream_devices)
            })
            .at(Nsid::ER)
            .at("args"),
            additional_errors: vec![],
        };

        result.handle(dbg!(msg.body.args.period.require_empty().at("args")));

        result.try_into()
    }
}

pub struct EndpointResponse {
    client: Arc<Client>,
}

impl EndpointResponse {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    pub async fn contain_device(&self, args: ContainDeviceArgs) -> Result<Response, Error> {
        self.client
            .contain_device(&args.device_id)
            .await
            .map_err(Error::custom)?;
        Ok(Response::new(StatusCode::Ok))
    }

    pub async fn stop_process(&self, _args: StopProcessArgs) -> Result<Response, Error> {
        todo!()
    }

    async fn consume_contain_device(
        &self,
        msg: Message<Headers, Command>,
    ) -> Result<Response, Error> {
        self.contain_device(msg.try_into()?).await
    }

    async fn consume_stop_process(
        &self,
        msg: Message<Headers, Command>,
    ) -> Result<Response, Error> {
        self.stop_process(msg.try_into()?).await
    }

    fn consume_delete_file<'a>(
        &'a self,
        msg: Message<Headers, Command>,
    ) -> BoxStream<'a, Response> {
        let cmd = match DeleteFileArgs::try_from(msg) {
            Ok(cmd) => cmd,
            Err(e) => {
                return stream_just(e.into());
            }
        };

        stream::select_all(
            cmd.target_devices.into_iter().map(move |device_id| {
                self.delete_file_from_device(device_id, cmd.file_path.clone())
            }),
        )
        .boxed()
    }

    fn delete_file_from_device<'a>(
        &'a self,
        aid: Aid,
        file_path: String,
    ) -> BoxStream<'a, Response> {
        stream::iter([Response::new(StatusCode::Processing)])
            .chain(stream::once(
                self.client
                    .delete_file(file_path.clone(), aid)
                    .map(|res| res.map(|_| Response::new(StatusCode::Ok)).into()),
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
    fn consume<'a>(&'a self, msg: Message<Headers, Command>) -> BoxStream<'a, Response> {
        match msg.body.as_action_target() {
            (Action::Contain, Target::Device(_)) => {
                stream::once(self.consume_contain_device(msg).map(Response::from)).boxed()
            }
            (Action::Delete, Target::File(_)) => self.consume_delete_file(msg),
            (Action::Stop, Target::Process(_)) => {
                stream::once(self.consume_stop_process(msg).map(Response::from)).boxed()
            }
            (action, target) => stream_just(Response::from(Error::validation(format!(
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
    fn consume<'a>(&'a self, msg: Message<Headers, Command>) -> BoxStream<'a, Response> {
        stream::once(self.consume_msg(msg).map(Response::from)).boxed()
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
