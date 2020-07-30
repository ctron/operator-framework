/*
 * Copyright (c) 2020 Jens Reimann and others.
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
use k8s_openapi::api::core::v1::{ConfigMap, Secret};
use k8s_openapi::ByteString;
use sha1::Sha1;
use std::collections::BTreeMap;

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

    pub fn track(&mut self, data: &[u8]) {
        self.sha.update(data);
    }

    pub fn current_hash(&self) -> String {
        self.sha.clone().digest().to_string()
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
        if let Some(ref data) = self.data {
            data.track_with(tracker);
        }
    }
}

impl Trackable for ConfigMap {
    fn track_with(&self, tracker: &mut ConfigTracker) {
        if let Some(ref data) = self.data {
            data.track_with(tracker);
        }
    }
}
