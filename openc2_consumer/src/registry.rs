use std::collections::{BTreeSet, HashMap, HashSet};

use async_trait::async_trait;
use futures::future::join_all;
use openc2::{
    Action, ActionTargets, Error, Feature, Headers, Message, Nsid, ProfileFeatures, StatusCode,
    TargetType, Value, Version,
    json::{Command, Response, Results, Target},
    target::Features,
};

use crate::Consume;

pub struct ConsumerToken(usize);

/// A registration of an OpenC2 consumer along with the action/target pairs it wishes to handle.
pub struct Registration {
    consumer: Box<dyn Consume + Send + Sync>,
    /// A map of the action targets this consumer wishes to handle, keyed by optional profile.
    actions: HashMap<Option<Nsid>, ActionTargets>,
}

impl Registration {
    pub fn new(consumer: impl Consume + Send + Sync + 'static) -> Self {
        Self {
            consumer: Box::new(consumer),
            actions: Default::default(),
        }
    }

    pub fn with_actions(
        mut self,
        actions: impl IntoIterator<Item = (Nsid, Action, TargetType<'static>)>,
    ) -> Self {
        for (nsid, action, target) in actions {
            self.actions
                .entry(Some(nsid))
                .or_default()
                .entry(action)
                .or_default()
                .insert(target);
        }
        self
    }

    pub fn with_actions_without_profile(
        mut self,
        actions: impl IntoIterator<Item = (Action, TargetType<'static>)>,
    ) -> Self {
        for (action, target) in actions {
            self.actions
                .entry(None)
                .or_default()
                .entry(action)
                .or_default()
                .insert(target);
        }
        self
    }

    /// Returns the profiles this consumer supports.
    /// This could be empty if the consumer only supports actions without profiles.
    pub fn profiles(&self) -> impl Iterator<Item = &Nsid> {
        self.actions.keys().flatten()
    }

    fn to_pairs(&self) -> impl Iterator<Item = (Action, TargetType<'static>)> {
        self.actions
            .values()
            .flatten()
            .flat_map(|(a, t)| t.iter().cloned().map(move |target| (*a, target)))
    }

    /// Checks if this registration matches the given action, target type, and profile.
    pub fn matches(&self, action: Action, target: &TargetType, profile: &Nsid) -> bool {
        let Some(entry) = self.actions.get(&Some(profile.clone())) else {
            return false;
        };
        entry
            .get(&action)
            .map(|set| set.contains(target))
            .unwrap_or(false)
    }

    pub fn query_features(&self, features: &Features) -> Result<Response, Error> {
        if features.contains(&Feature::RateLimit) {
            return Err(
                Error::not_implemented("rate limit feature is not implemented").at("features"),
            );
        }

        let mut results = Results::default();
        if features.contains(&Feature::Profiles) {
            results.profiles = self.actions.keys().flatten().cloned().collect();
        }

        if features.contains(&Feature::Versions) {
            results.versions = [Version::new(2, 0)].into_iter().collect();
        }

        if features.contains(&Feature::Pairs) {
            results.pairs = Some(self.actions.values().cloned().fold(
                ActionTargets::new(),
                |mut acc, at| {
                    for (a, t) in &at {
                        for target in t {
                            acc.entry(*a).or_default().insert(target.clone());
                        }
                    }
                    acc
                },
            ));

            results.extensions = self
                .actions
                .iter()
                .filter_map(|(k, v)| {
                    Some((
                        k.clone()?,
                        Value::from_typed(&ProfileFeatures { pairs: v.clone() }).unwrap(),
                    ))
                })
                .collect();
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

        for pair in registration.to_pairs() {
            self.by_pair.entry(pair).or_default().insert(idx);
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
        for pair in entry.to_pairs() {
            if let Some(set) = self.by_pair.get_mut(&pair) {
                set.remove(&token.0);
                if set.is_empty() {
                    self.by_pair.remove(&pair);
                }
            }
        }
        Some(entry)
    }

    pub fn profiles(&self) -> HashSet<&Nsid> {
        self.consumers
            .iter()
            .filter_map(|c| c.as_ref())
            .flat_map(|c| c.profiles())
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
        let mut actions: HashMap<Option<Nsid>, ActionTargets> = HashMap::new();

        for (profile, acts) in value.consumers.iter().flatten().flat_map(|c| &c.actions) {
            let profile_entry = actions.entry(profile.clone()).or_default();
            for (action, targets) in acts {
                profile_entry
                    .entry(*action)
                    .or_default()
                    .extend(targets.iter().cloned());
            }
        }

        Self {
            actions,
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
                results.profiles = self.profiles().into_iter().cloned().collect();
            }

            if features.contains(&Feature::Versions) {
                results.versions = [Version::new(2, 0)].into_iter().collect();
            }

            if features.contains(&Feature::Pairs) {
                results.pairs = Some(self.pairs());

                let mut profiles: HashMap<_, ActionTargets> = HashMap::new();
                for consumer in self.consumers.iter().flatten() {
                    for (profile, actions) in &consumer.actions {
                        let Some(profile) = profile else {
                            continue;
                        };
                        let profile_entry = profiles.entry(profile.clone()).or_default();
                        for (action, target) in actions {
                            profile_entry
                                .entry(*action)
                                .or_default()
                                .extend(target.clone());
                        }
                    }
                }

                results = results
                    .with_extensions(
                        profiles
                            .into_iter()
                            .map(|(ap, pairs)| (ap, ProfileFeatures { pairs })),
                    )
                    .map_err(|e| {
                        Error::custom(format!("unable to serialize profile-specific pairs: {e}"))
                    })?;
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
            consumers.retain(|consumer| consumer.matches(action, &target_type, profile));
        }

        if consumers.is_empty() {
            return Err(Error::not_implemented(format!(
                "No consumer for action '{action}' and target type '{target_type:?}' matches profile '{:?}'",
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
