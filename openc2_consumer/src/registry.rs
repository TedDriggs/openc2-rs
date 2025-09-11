use std::collections::{BTreeSet, HashMap, HashSet};

use async_trait::async_trait;
use futures::future::join_all;
use openc2::{
    Action, ActionTargets, Error, Feature, Headers, Message, Nsid, StatusCode, TargetType, Version,
    json::{Command, Response, Results, Target},
    target::Features,
};

use crate::Consume;

pub struct ConsumerToken(usize);

pub struct Registration {
    consumer: Box<dyn Consume + Send + Sync>,
    actions: HashSet<(Action, TargetType<'static>)>,
    profiles: HashSet<Nsid>,
}

impl Registration {
    pub fn new(consumer: impl Consume + Send + Sync + 'static) -> Self {
        Self {
            consumer: Box::new(consumer),
            actions: Default::default(),
            profiles: Default::default(),
        }
    }

    pub fn with_actions(
        mut self,
        actions: impl IntoIterator<Item = (Action, TargetType<'static>)>,
    ) -> Self {
        self.actions = actions.into_iter().collect();
        self
    }

    pub fn with_extended_actions(
        mut self,
        actions: impl IntoIterator<Item = (Action, TargetType<'static>)>,
    ) -> Self {
        self.actions.extend(actions);

        self
    }

    pub fn with_profile(mut self, profile: Nsid) -> Self {
        self.profiles.clear();
        self.profiles.insert(profile);
        self
    }

    pub fn with_profiles(mut self, profiles: impl IntoIterator<Item = Nsid>) -> Self {
        self.profiles.clear();
        self.profiles.extend(profiles);
        self
    }

    pub fn with_extended_profiles(mut self, profiles: impl IntoIterator<Item = Nsid>) -> Self {
        self.profiles.extend(profiles);
        self
    }

    pub fn query_features(&self, features: &Features) -> Result<Response, Error> {
        if features.contains(&Feature::RateLimit) {
            return Err(
                Error::not_implemented("rate limit feature is not implemented").at("features"),
            );
        }

        let mut results = Results::default();
        if features.contains(&Feature::Profiles) {
            results.profiles = self.profiles.iter().cloned().collect();
        }

        if features.contains(&Feature::Versions) {
            results.versions = [Version::new(2, 0)].into_iter().collect();
        }

        if features.contains(&Feature::Pairs) {
            results.pairs = Some(self.actions.iter().cloned().fold(
                ActionTargets::new(),
                |mut acc, (a, t)| {
                    acc.entry(a).or_default().insert(t);
                    acc
                },
            ));
        }

        Ok(results.into())
    }
}

#[async_trait]
impl Consume for Registration {
    async fn consume(&self, msg: Message<Headers, Command>) -> Result<Response, Error> {
        if let (Action::Query, Target::Features(features)) = msg.body.as_action_target() {
            return self.query_features(features);
        }

        self.consumer.consume(msg).await
    }
}

/// An OpenC2 consumer made up of more specific consumers.
#[derive(Default)]
pub struct Registry {
    consumers: Vec<Option<Registration>>,
    by_pair: HashMap<(Action, TargetType<'static>), BTreeSet<usize>>,
}

impl Registry {
    /// Register an OpenC2 consumer.
    ///
    /// Returns a token that can be used to unregister the consumer.
    pub fn insert(&mut self, registration: impl Into<Registration>) -> ConsumerToken {
        let idx = self.consumers.len();
        let registration = registration.into();

        for action in registration.actions.iter().cloned() {
            self.by_pair.entry(action).or_default().insert(idx);
        }

        self.consumers.push(Some(registration));

        ConsumerToken(idx)
    }

    fn get_matching<'a>(
        &'a self,
        pair: &(Action, TargetType<'a>),
    ) -> impl Iterator<Item = &'a Registration> + use<'a> {
        let entry = self.by_pair.get(pair);
        entry.into_iter().flat_map(move |indices| {
            indices
                .iter()
                .filter_map(|&idx| self.consumers[idx].as_ref())
        })
    }

    /// Unregister an OpenC2 consumer. This will not drop any in-progress requests.
    pub fn remove(&mut self, token: ConsumerToken) -> Option<Registration> {
        let entry = self.consumers.get_mut(token.0)?.take()?;
        for pair in &entry.actions {
            if let Some(set) = self.by_pair.get_mut(pair) {
                set.remove(&token.0);
                if set.is_empty() {
                    self.by_pair.remove(pair);
                }
            }
        }
        Some(entry)
    }

    pub fn profiles(&self) -> HashSet<Nsid> {
        self.consumers
            .iter()
            .filter_map(|c| c.as_ref())
            .flat_map(|c| c.profiles.iter().cloned())
            .collect()
    }

    pub fn pairs(&self) -> ActionTargets {
        let mut pairs = ActionTargets::new();
        for (action, target) in self.by_pair.keys().cloned() {
            pairs.entry(action).or_default().insert(target);
        }
        pairs
    }
}

impl FromIterator<Registration> for Registry {
    fn from_iter<T: IntoIterator<Item = Registration>>(iter: T) -> Self {
        let mut registry = Self::default();
        for registration in iter {
            registry.insert(registration);
        }
        registry
    }
}

impl From<Registry> for Registration {
    fn from(value: Registry) -> Self {
        Self {
            actions: value.by_pair.keys().cloned().collect(),
            profiles: value.profiles(),
            consumer: Box::new(value),
        }
    }
}

#[async_trait]
impl Consume for Registry {
    async fn consume(&self, msg: Message<Headers, Command>) -> Result<Response, Error> {
        if msg.body.action == Action::Query
            && let Target::Features(features) = &msg.body.target
        {
            if features.contains(&Feature::RateLimit) {
                return Err(
                    Error::not_implemented("rate limit feature is not implemented").at("features"),
                );
            }

            let mut results = Results::default();
            if features.contains(&Feature::Profiles) {
                results.profiles = self.profiles().into_iter().collect();
            }

            if features.contains(&Feature::Versions) {
                results.versions = [Version::new(2, 0)].into_iter().collect();
            }

            if features.contains(&Feature::Pairs) {
                results.pairs = Some(self.pairs());
            }

            return Ok(Response::new(StatusCode::Ok).with_results(results));
        }

        let action = msg.body.action;
        let target_type = msg.body.target.kind();
        let mut consumers = self
            .get_matching(&(action, target_type.clone()))
            .collect::<Vec<_>>();

        if consumers.is_empty() {
            return Err(Error::not_implemented_pair(action, &target_type));
        }

        if let Some(profile) = &msg.body.profile {
            consumers.retain(|consumer| consumer.profiles.contains(profile));
        }

        if consumers.is_empty() {
            return Err(Error::not_implemented(format!(
                "No consumer found for action '{action}' and target type '{target_type:?}' with profile '{:?}'",
                msg.body.profile
            )));
        }

        let futures = consumers
            .into_iter()
            .map(|consumer| consumer.consume(msg.clone()));
        let results: Vec<Result<Response, Error>> = join_all(futures).await;
        // TODO figure out how to combine multiple responses
        return results
            .into_iter()
            .next()
            .expect("at least one consumer exists");
    }
}
