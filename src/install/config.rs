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
    fn append_string<S>(&mut self, key: S, value: T)
    where
        S: ToString;
}

impl<T: Into<Vec<u8>>> AppendString<T> for Secret {
    fn append_string<S>(&mut self, key: S, value: T)
    where
        S: ToString,
    {
        self.data
            .use_or_create(|data| data.insert(key.to_string(), ByteString(value.into())));
    }
}

impl<T: Into<String>> AppendString<T> for ConfigMap {
    fn append_string<S>(&mut self, key: S, value: T)
    where
        S: ToString,
    {
        self.data
            .use_or_create(|data| data.insert(key.to_string(), value.into()));
    }
}

pub trait AppendBinary<T> {
    fn append_binary<S>(&mut self, key: S, value: T)
    where
        S: ToString;
}

impl<T: Into<Vec<u8>>> AppendBinary<T> for Secret {
    fn append_binary<S>(&mut self, key: S, value: T)
    where
        S: ToString,
    {
        self.data
            .use_or_create(|data| data.insert(key.to_string(), ByteString(value.into())));
    }
}

impl<T: Into<Vec<u8>>> AppendBinary<T> for ConfigMap {
    fn append_binary<S>(&mut self, key: S, value: T)
    where
        S: ToString,
    {
        self.binary_data
            .use_or_create(|data| data.insert(key.to_string(), ByteString(value.into())));
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_cm_string() {
        let mut cm: ConfigMap = Default::default();
        cm.append_string("foo", "bar");
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
