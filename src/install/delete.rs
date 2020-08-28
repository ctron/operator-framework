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

use async_trait::async_trait;

use either::Either::{Left, Right};
use kube::api::{DeleteParams, Meta};
use kube::{Api, Error};
use serde::de::DeserializeOwned;

use futures::future::FutureExt;

#[async_trait]
pub trait DeleteOptionally {
    /// Optionally delete a resource. If the resource was already gone, this is not treated as an error.
    ///
    /// The function will return `true` if the resource was deleted (or already gone) and `false` if
    /// the resource is being delete. All other errors are returned unmodified.
    async fn delete_optionally(&self, name: &str, dp: &DeleteParams) -> Result<bool, kube::Error>;
}

#[async_trait]
impl<K: Clone + DeserializeOwned + Meta> DeleteOptionally for Api<K> {
    async fn delete_optionally(&self, name: &str, dp: &DeleteParams) -> Result<bool, kube::Error> {
        self.delete(name, dp)
            .map(|future| {
                future
                    .map(|either| match either {
                        Left(_) => false,
                        Right(_) => true,
                    })
                    .or_else(|err| match err {
                        Error::Api(cause) if cause.reason == "NotFound" => Ok(true),
                        _ => Err(err),
                    })
            })
            .await
    }
}
