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

use crate::utils::UseOrCreate;

pub trait AppendString<T> {
    fn insert_string<S, P>(&mut self, key: S, keep_existing: bool, provider: P)
    where
        S: ToString,
        P: FnOnce() -> T;

    fn append_string<S>(&mut self, key: S, value: T)
    where
        S: ToString,
    {
        self.insert_string(key, false, || value);
    }

    fn init_string_from<S, P>(&mut self, key: S, provider: P)
    where
        S: ToString,
        P: FnOnce() -> T,
    {
        self.insert_string(key, true, provider);
    }

    fn init_string<S>(&mut self, key: S, value: T)
    where
        S: ToString,
    {
        self.insert_string(key, true, || value);
    }
}

impl<T: Into<String>> AppendString<T> for Secret {
    fn insert_string<S, P>(&mut self, key: S, keep_existing: bool, provider: P)
    where
        S: ToString,
        P: FnOnce() -> T,
    {
        self.data.use_or_create(|data| {
            if keep_existing {
                let entry = data.entry(key.to_string());
                entry.or_insert(ByteString(provider().into().into_bytes()));
            } else {
                data.insert(key.to_string(), ByteString(provider().into().into_bytes()));
            }
        });
    }
}

impl<T: Into<String>> AppendString<T> for ConfigMap {
    fn insert_string<S, P>(&mut self, key: S, keep_existing: bool, provider: P)
    where
        S: ToString,
        P: FnOnce() -> T,
    {
        self.data.use_or_create(|data| {
            if keep_existing {
                let entry = data.entry(key.to_string());
                entry.or_insert(provider().into());
            } else {
                data.insert(key.to_string(), provider().into());
            }
        });
    }
}

pub trait AppendBinary<T> {
    fn insert_binary<S, P>(&mut self, key: S, keep_existing: bool, provider: P)
    where
        S: ToString,
        P: FnOnce() -> T;

    fn append_binary<S>(&mut self, key: S, value: T)
    where
        S: ToString,
    {
        self.insert_binary(key, false, || value);
    }

    fn init_binary_from<S, P>(&mut self, key: S, provider: P)
    where
        S: ToString,
        P: FnOnce() -> T,
    {
        self.insert_binary(key, true, provider);
    }

    fn init_binary<S>(&mut self, key: S, value: T)
    where
        S: ToString,
    {
        self.insert_binary(key, true, || value);
    }
}

impl<T: Into<Vec<u8>>> AppendBinary<T> for Secret {
    fn insert_binary<S, P>(&mut self, key: S, keep_existing: bool, provider: P)
    where
        S: ToString,
        P: FnOnce() -> T,
    {
        self.data.use_or_create(|data| {
            if keep_existing {
                let entry = data.entry(key.to_string());
                entry.or_insert(ByteString(provider().into()));
            } else {
                data.insert(key.to_string(), ByteString(provider().into()));
            }
        });
    }
}

impl<T: Into<Vec<u8>>> AppendBinary<T> for ConfigMap {
    fn insert_binary<S, P>(&mut self, key: S, keep_existing: bool, provider: P)
    where
        S: ToString,
        P: FnOnce() -> T,
    {
        self.binary_data.use_or_create(|data| {
            if keep_existing {
                let entry = data.entry(key.to_string());
                entry.or_insert(ByteString(provider().into()));
            } else {
                data.insert(key.to_string(), ByteString(provider().into()));
            }
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_cm_string_append() {
        let mut cm: ConfigMap = Default::default();
        cm.append_string("foo", "bar");

        let mut expected = BTreeMap::new();
        expected.insert("foo".into(), "bar".into());
        assert_eq!(cm.data, Some(expected));

        cm.append_string("foo", "bar2");
        let mut expected = BTreeMap::new();
        expected.insert("foo".into(), "bar2".into());
        assert_eq!(cm.data, Some(expected));
    }

    #[test]
    fn test_cm_string_init() {
        let mut cm: ConfigMap = Default::default();
        cm.append_string("foo", "bar");

        let mut expected = BTreeMap::new();
        expected.insert("foo".into(), "bar".into());
        assert_eq!(cm.data, Some(expected.clone()));

        cm.init_string("foo", "bar2");
        assert_eq!(cm.data, Some(expected));
    }

    #[test]
    fn test_secret_string() {
        let mut secret: Secret = Default::default();
        secret.append_string("foo", "bar");
    }

    #[test]
    fn test_cm_binary() {
        let mut cm: ConfigMap = Default::default();
        let data = [1u8, 2u8, 3u8];
        cm.append_binary("foo", &data[..]);
    }

    #[test]
    fn test_cm_bigger_binary() {
        let mut cm: ConfigMap = Default::default();
        let data = [0u8; 100];
        cm.append_binary("foo", &data[..]);
    }

    #[test]
    fn test_secret_binary() {
        let mut secret: Secret = Default::default();
        let data = [1u8, 2u8, 3u8];
        secret.append_binary("foo", data.clone());
    }
}
