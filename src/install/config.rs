/**
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

pub trait AppendData<T> {
    fn append_data<S>(&mut self, key: S, value: T)
    where
        S: ToString;
}

impl<T: Into<Vec<u8>>> AppendData<T> for Secret {
    fn append_data<S>(&mut self, key: S, value: T)
    where
        S: ToString,
    {
        self.data
            .use_or_create(|data| data.insert(key.to_string(), ByteString(value.into())));
    }
}

impl<T: Into<String>> AppendData<T> for ConfigMap {
    fn append_data<S>(&mut self, key: S, value: T)
    where
        S: ToString,
    {
        self.data
            .use_or_create(|data| data.insert(key.to_string(), value.into()));
    }
}
