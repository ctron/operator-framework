/*
 * Copyright (c) 2020, 2022 Jens Reimann and others.
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
use k8s_openapi::{
    api::core::v1::{ConfigMap, Secret},
    ByteString,
};
use sha1::{Digest, Sha1};
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

/// Tracking content changes of configurations.
///
/// This is useful for things like ConfigMaps and Secrets, where a change in content
/// should trigger a redeployment. The config tracker keeps an internal hash, which,
/// for example, can be applied to the annotation of a PodSpec. A change in content will
/// result a changed hash, and thus a change in the PodSpec, resulting in a redeployment.
pub struct ConfigTracker {
    sha: Sha1,
}

pub trait Trackable {
    fn track_with(&self, tracker: &mut ConfigTracker);
}

impl ConfigTracker {
    pub fn new() -> Self {
        ConfigTracker { sha: Sha1::new() }
    }

    pub fn track<D>(&mut self, data: D)
    where
        D: AsRef<[u8]>,
    {
        self.sha.update(data.as_ref());
    }

    pub fn current_hash(&self) -> String {
        format!("{:x}", self.sha.clone().finalize())
    }

    /// Freeze the current tracker state and return it.
    pub fn freeze(self) -> TrackerState {
        TrackerState(self.current_hash())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackerState(pub String);

impl AsRef<[u8]> for TrackerState {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl From<TrackerState> for String {
    fn from(state: TrackerState) -> Self {
        state.0
    }
}

impl Display for TrackerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Trackable for TrackerState {
    fn track_with(&self, tracker: &mut ConfigTracker) {
        tracker.track(self.0.as_bytes())
    }
}

impl<K> Trackable for BTreeMap<K, String> {
    fn track_with(&self, tracker: &mut ConfigTracker) {
        for (_, v) in self.iter() {
            tracker.track(v.as_bytes());
        }
    }
}

impl<K> Trackable for BTreeMap<K, ByteString> {
    fn track_with(&self, tracker: &mut ConfigTracker) {
        for (_, v) in self.iter() {
            tracker.track(v.0.as_slice());
        }
    }
}

impl Trackable for Secret {
    fn track_with(&self, tracker: &mut ConfigTracker) {
        if let Some(data) = &self.data {
            data.track_with(tracker);
        }
    }
}

impl Trackable for ConfigMap {
    fn track_with(&self, tracker: &mut ConfigTracker) {
        if let Some(data) = &self.data {
            data.track_with(tracker);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let tracker = ConfigTracker::new();
        assert_eq!(
            "da39a3ee5e6b4b0d3255bfef95601890afd80709",
            tracker.current_hash()
        );
    }
}
