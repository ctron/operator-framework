/*
 * Copyright (c) 2020, 2021 Jens Reimann and others.
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
use futures::future::FutureExt;
use kube::{
    api::{DeleteParams, Preconditions},
    Api, Error, Resource,
};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[async_trait]
pub trait Delete<R: Send> {
    /// Optionally delete a resource. If the resource was already gone, this is not treated as an error.
    ///
    /// The function will return `true` if the resource was deleted (or already gone) and `false` if
    /// the resource is being delete. All other errors are returned unmodified.
    async fn delete_optionally(&self, name: &str, dp: &DeleteParams) -> Result<bool, kube::Error>;

    async fn delete_conditionally<F, E>(&self, name: &str, f: F) -> Result<bool, E>
    where
        F: FnOnce(&R) -> Result<bool, E> + Send,
        E: From<kube::Error>;
}

#[async_trait]
impl<K> Delete<K> for Api<K>
where
    K: Resource + Clone + DeserializeOwned + Send + Debug,
{
    async fn delete_optionally(&self, name: &str, dp: &DeleteParams) -> Result<bool, kube::Error> {
        Ok(self
            .delete(name, dp)
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
            .await?)
    }

    async fn delete_conditionally<F, E>(&self, name: &str, f: F) -> Result<bool, E>
    where
        F: FnOnce(&K) -> Result<bool, E> + Send,
        E: From<kube::Error>,
    {
        let resource = match self.get(name).await {
            Err(Error::Api(cause)) if cause.reason == "NotFound" => return Ok(false),
            result => result?,
        };

        if f(&resource)? {
            let dp = DeleteParams {
                preconditions: Some(Preconditions {
                    resource_version: resource.meta().resource_version.as_ref().cloned(),
                    uid: resource.meta().uid.as_ref().cloned(),
                }),
                ..Default::default()
            };

            self.delete_optionally(name, &dp).await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }
}
