/*
 * Copyright (c) 2021 Jens Reimann and others.
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Eclipse Public License 2.0 which is available at
 * http://www.eclipse.org/legal/epl-2.0
 *
 * SPDX-License-Identifier: EPL-2.0
 */

mod k8s;

pub use k8s::*;

use crate::utils::UseOrCreate;
use chrono::{DateTime, Utc};
use std::fmt::{Display, Formatter};

/// The state of the condition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    True,
    False,
    Unknown,
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::True => write!(f, "True"),
            Self::False => write!(f, "False"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl From<State> for StateDetails {
    fn from(state: State) -> Self {
        Self {
            state,
            reason: None,
            message: None,
            observed_generation: None,
        }
    }
}

/// Help building a condition state.
pub trait StateBuilder: Sized {
    fn with_reason<S>(self, reason: S) -> StateDetails
    where
        S: Into<String>,
    {
        self.with_reason_opt(Some(reason.into()))
    }

    fn with_message<S>(self, message: S) -> StateDetails
    where
        S: Into<String>,
    {
        self.with_message_opt(Some(message.into()))
    }

    fn with_reason_opt<S>(self, reason: S) -> StateDetails
    where
        S: Into<Option<String>>;

    fn with_message_opt<S>(self, message: S) -> StateDetails
    where
        S: Into<Option<String>>;

    fn with_observed<G>(self, observed: G) -> StateDetails
    where
        G: Into<Option<i64>>;
}

/// Details of the condition state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateDetails {
    pub state: State,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub observed_generation: Option<i64>,
}

impl StateBuilder for State {
    fn with_reason_opt<S>(self, reason: S) -> StateDetails
    where
        S: Into<Option<String>>,
    {
        StateDetails::from(self).with_reason_opt(reason)
    }

    fn with_message_opt<S>(self, message: S) -> StateDetails
    where
        S: Into<Option<String>>,
    {
        StateDetails::from(self).with_message_opt(message)
    }

    fn with_observed<G>(self, observed: G) -> StateDetails
    where
        G: Into<Option<i64>>,
    {
        StateDetails::from(self).with_observed(observed)
    }
}

impl StateBuilder for StateDetails {
    fn with_reason_opt<S>(mut self, reason: S) -> StateDetails
    where
        S: Into<Option<String>>,
    {
        self.reason = reason.into();
        self
    }

    fn with_message_opt<S>(mut self, message: S) -> StateDetails
    where
        S: Into<Option<String>>,
    {
        self.message = message.into();
        self
    }

    fn with_observed<G>(mut self, observed: G) -> StateDetails
    where
        G: Into<Option<i64>>,
    {
        self.observed_generation = observed.into();
        self
    }
}

/// Trait to allow universal access to the conditions
pub trait Condition {
    fn state(&self) -> State;
    fn set_state(&mut self, state: State);

    fn r#type(&self) -> &str;

    fn message(&self) -> Option<&str>;
    fn set_message<S>(&mut self, message: S)
    where
        S: Into<Option<String>>;

    fn reason(&self) -> Option<&str>;
    fn set_reason<S>(&mut self, reason: S)
    where
        S: Into<Option<String>>;

    fn last_probe_time(&self) -> Option<chrono::DateTime<chrono::Utc>>;
    fn set_last_probe_time<T>(&mut self, time: T)
    where
        T: Into<Option<chrono::DateTime<chrono::Utc>>>;

    fn last_transition_time(&self) -> Option<chrono::DateTime<chrono::Utc>>;
    fn set_last_transition_time<T>(&mut self, time: T)
    where
        T: Into<Option<chrono::DateTime<chrono::Utc>>>;

    fn observed_generation(&self) -> Option<i64>;
    fn set_observed_generation<S>(&mut self, observed_generation: S)
    where
        S: Into<Option<i64>>;

    fn from(
        r#type: String,
        state: State,
        reason: Option<String>,
        message: Option<String>,
        observed_generation: Option<i64>,
        now: DateTime<Utc>,
    ) -> Self;
}

#[macro_export]
macro_rules! condition {
    ($n:ty) => {
        impl $crate::conditions::Condition for $n {
            fn from(
                r#type: String,
                state: $crate::conditions::State,
                reason: Option<String>,
                message: Option<String>,
                _observed_generation: Option<i64>,
                now: chrono::DateTime<chrono::Utc>,
            ) -> Self {
                let now = k8s_openapi::apimachinery::pkg::apis::meta::v1::Time(now);
                Self {
                    last_transition_time: Some(now),
                    reason,
                    message,
                    status: state.to_string(),
                    type_: r#type,
                }
            }

            fn last_probe_time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                None
            }

            fn set_last_probe_time<T>(&mut self, _: T)
            where
                T: Into<Option<chrono::DateTime<chrono::Utc>>>,
            {
            }

            $crate::condition!($n[common]);
            $crate::condition!($n[opt]);
        }
    };
    ($n:ty[core]) => {
        impl $crate::conditions::Condition for $n {
            fn from(
                r#type: String,
                state: $crate::conditions::State,
                reason: Option<String>,
                message: Option<String>,
                observed_generation: Option<i64>,
                now: chrono::DateTime<chrono::Utc>,
            ) -> Self {
                let now = k8s_openapi::apimachinery::pkg::apis::meta::v1::Time(now);
                Self {
                    last_transition_time: now,
                    reason: reason.unwrap_or_default(),
                    message: message.unwrap_or_default(),
                    observed_generation,
                    status: state.to_string(),
                    type_: r#type,
                }
            }

            fn last_probe_time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                None
            }

            fn set_last_probe_time<T>(&mut self, _: T)
            where
                T: Into<Option<chrono::DateTime<chrono::Utc>>>,
            {
            }

            $crate::condition!($n[common]);
            $crate::condition!($n[non_opt]);
        }
    };
    ($n:ty [probe]) => {
        impl $crate::conditions::Condition for $n {
            fn from(
                r#type: String,
                state: $crate::conditions::State,
                reason: Option<String>,
                message: Option<String>,
                _observed_generation: Option<i64>,
                now: chrono::DateTime<chrono::Utc>,
            ) -> Self {
                let now = k8s_openapi::apimachinery::pkg::apis::meta::v1::Time(now);
                Self {
                    last_probe_time: Some(now.clone()),
                    last_transition_time: Some(now),
                    reason,
                    message,
                    status: state.to_string(),
                    type_: r#type,
                }
            }

            fn last_probe_time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                self.last_probe_time.as_ref().map(|t| t.0)
            }

            fn set_last_probe_time<T>(&mut self, time: T)
            where
                T: Into<Option<chrono::DateTime<chrono::Utc>>>,
            {
                self.last_probe_time = time
                    .into()
                    .map(k8s_openapi::apimachinery::pkg::apis::meta::v1::Time);
            }

            $crate::condition!($n[common]);
            $crate::condition!($n[opt]);
        }
    };

    ($n:ty [common]) => {
        fn state(&self) -> $crate::conditions::State {
            use std::ops::Deref;
            match self.status.deref() {
                "True" => $crate::conditions::State::True,
                "False" => $crate::conditions::State::False,
                _ => $crate::conditions::State::Unknown,
            }
        }

        fn set_state(&mut self, state: $crate::conditions::State) {
            self.status = state.to_string();
        }

        fn r#type(&self) -> &str {
            &self.type_
        }
    };
    ($n:ty [non_opt]) => {
        fn message(&self) -> Option<&str> {
            Some(&self.message)
        }

        fn set_message<S>(&mut self, message: S)
        where
            S: Into<Option<String>>,
        {
            self.message = message.into().unwrap_or_default();
        }

        fn reason(&self) -> Option<&str> {
            Some(&self.reason)
        }

        fn set_reason<S>(&mut self, reason: S)
        where
            S: Into<Option<String>>,
        {
            self.reason = reason.into().unwrap_or_default();
        }

        fn last_transition_time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
            Some(self.last_transition_time.0)
        }

        fn set_last_transition_time<T>(&mut self, time: T)
        where
            T: Into<Option<chrono::DateTime<chrono::Utc>>>,
        {
            self.last_transition_time = k8s_openapi::apimachinery::pkg::apis::meta::v1::Time(
                time.into().unwrap_or_else(|| chrono::Utc::now()),
            );
        }

        fn observed_generation(&self) -> Option<i64> {
            self.observed_generation
        }
        fn set_observed_generation<S>(&mut self, observed_generation: S)
        where
            S: Into<Option<i64>>,
        {
            self.observed_generation = observed_generation.into();
        }
    };
    ($n:ty [opt]) => {
        fn message(&self) -> Option<&str> {
            self.message.as_deref()
        }

        fn set_message<S>(&mut self, message: S)
        where
            S: Into<Option<String>>,
        {
            self.message = message.into();
        }

        fn reason(&self) -> Option<&str> {
            self.reason.as_deref()
        }

        fn set_reason<S>(&mut self, reason: S)
        where
            S: Into<Option<String>>,
        {
            self.reason = reason.into();
        }
        fn last_transition_time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
            self.last_transition_time.as_ref().map(|t| t.0)
        }

        fn set_last_transition_time<T>(&mut self, time: T)
        where
            T: Into<Option<chrono::DateTime<chrono::Utc>>>,
        {
            self.last_transition_time = time
                .into()
                .map(k8s_openapi::apimachinery::pkg::apis::meta::v1::Time);
        }

        fn observed_generation(&self) -> Option<i64> {
            None
        }

        fn set_observed_generation<S>(&mut self, _: S)
        where
            S: Into<Option<i64>>,
        {
        }
    };
}

pub trait Conditions {
    fn update_condition<S, D>(&mut self, r#type: S, state: D)
    where
        S: AsRef<str>,
        D: Into<StateDetails>,
    {
        self.update_condition_on(r#type, state, Utc::now())
    }

    fn update_condition_on<S, D, DT>(&mut self, r#type: S, state: D, now: DT)
    where
        S: AsRef<str>,
        D: Into<StateDetails>,
        DT: Into<DateTime<Utc>>;
}

impl<C> Conditions for Option<Vec<C>>
where
    C: Condition,
{
    fn update_condition_on<S, D, DT>(&mut self, r#type: S, state: D, now: DT)
    where
        S: AsRef<str>,
        D: Into<StateDetails>,
        DT: Into<DateTime<Utc>>,
    {
        self.use_or_create(|conditions| conditions.update_condition_on(r#type, state, now));
    }
}

impl<C> Conditions for Vec<C>
where
    C: Condition,
{
    fn update_condition_on<S, D, DT>(&mut self, r#type: S, state: D, now: DT)
    where
        S: AsRef<str>,
        D: Into<StateDetails>,
        DT: Into<DateTime<Utc>>,
    {
        let info = state.into();
        let now = now.into();

        for condition in self.into_iter() {
            if condition.r#type() == r#type.as_ref() {
                if condition.state() != info.state {
                    condition.set_last_transition_time(now);
                    condition.set_state(info.state);
                }
                condition.set_last_probe_time(now);
                condition.set_reason(info.reason);
                condition.set_message(info.message);
                condition.set_observed_generation(info.observed_generation);

                return;
            }
        }

        // did not find entry so far

        self.push(C::from(
            r#type.as_ref().to_string(),
            info.state,
            info.reason,
            info.message,
            info.observed_generation,
            now,
        ));
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::utils::UseOrCreate;
    use k8s_openapi::api::batch::v1::*;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;

    #[test]
    fn test_basic() {
        let mut job = Job {
            ..Default::default()
        };

        job.status.use_or_create(|status| {
            assert_eq!(status.conditions, None);

            // initial update

            let now = Utc::now();

            status
                .conditions
                .update_condition_on("Ready", State::True, now);

            assert_eq!(
                status.conditions,
                Some(vec![JobCondition {
                    type_: "Ready".into(),
                    status: "True".into(),
                    last_probe_time: Some(Time(now)),
                    last_transition_time: Some(Time(now)),
                    ..Default::default()
                }])
            );

            // second update, no change

            let now_2 = Utc::now();

            status
                .conditions
                .update_condition_on("Ready", State::True, now_2);

            assert_eq!(
                status.conditions,
                Some(vec![JobCondition {
                    type_: "Ready".into(),
                    status: "True".into(),
                    last_probe_time: Some(Time(now_2)),
                    last_transition_time: Some(Time(now)),
                    ..Default::default()
                }])
            );

            // third update, change

            let now = Utc::now();

            status.conditions.update_condition_on(
                "Ready",
                State::False
                    .with_reason("NotReady")
                    .with_message("Something failed"),
                now,
            );

            assert_eq!(
                status.conditions,
                Some(vec![JobCondition {
                    type_: "Ready".into(),
                    status: "False".into(),
                    reason: Some("NotReady".into()),
                    message: Some("Something failed".into()),
                    last_probe_time: Some(Time(now)),
                    last_transition_time: Some(Time(now)),
                    ..Default::default()
                }])
            );

            // fourth update, back to ok

            let now = Utc::now();

            status
                .conditions
                .update_condition_on("Ready", State::True, now);

            assert_eq!(
                status.conditions,
                Some(vec![JobCondition {
                    type_: "Ready".into(),
                    status: "True".into(),
                    last_probe_time: Some(Time(now)),
                    last_transition_time: Some(Time(now)),
                    ..Default::default()
                }])
            );
        });
    }
    #[test]
    fn test_multi() {
        let mut job = Job {
            ..Default::default()
        };

        job.status.use_or_create(|status| {
            assert_eq!(status.conditions, None);

            // initial update

            let now = Utc::now();

            status
                .conditions
                .update_condition_on("Ready", State::True, now);
            status
                .conditions
                .update_condition_on("Foo", State::True, now);
            status
                .conditions
                .update_condition_on("Bar", State::True, now);

            assert_eq!(
                status.conditions,
                Some(vec![
                    JobCondition {
                        type_: "Ready".into(),
                        status: "True".into(),
                        last_probe_time: Some(Time(now)),
                        last_transition_time: Some(Time(now)),
                        ..Default::default()
                    },
                    JobCondition {
                        type_: "Foo".into(),
                        status: "True".into(),
                        last_probe_time: Some(Time(now)),
                        last_transition_time: Some(Time(now)),
                        ..Default::default()
                    },
                    JobCondition {
                        type_: "Bar".into(),
                        status: "True".into(),
                        last_probe_time: Some(Time(now)),
                        last_transition_time: Some(Time(now)),
                        ..Default::default()
                    }
                ])
            );

            let now_2 = Utc::now();

            status
                .conditions
                .update_condition_on("Foo", State::False, now_2);

            assert_eq!(
                status.conditions,
                Some(vec![
                    JobCondition {
                        type_: "Ready".into(),
                        status: "True".into(),
                        last_probe_time: Some(Time(now)),
                        last_transition_time: Some(Time(now)),
                        ..Default::default()
                    },
                    JobCondition {
                        type_: "Foo".into(),
                        status: "False".into(),
                        last_probe_time: Some(Time(now_2)),
                        last_transition_time: Some(Time(now_2)),
                        ..Default::default()
                    },
                    JobCondition {
                        type_: "Bar".into(),
                        status: "True".into(),
                        last_probe_time: Some(Time(now)),
                        last_transition_time: Some(Time(now)),
                        ..Default::default()
                    }
                ])
            );
        });
    }
}
