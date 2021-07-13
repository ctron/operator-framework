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
use anyhow::Result;

use kube::{
    api::{ObjectMeta, PostParams},
    Api, Error, Resource,
};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

/// Create or update a Kubernetes resource.
pub async fn create_or_update_by<T, S1, S2, C, F, E, Eq>(
    api: &Api<T>,
    namespace: Option<S1>,
    name: S2,
    creator: C,
    eq: Eq,
    mutator: F,
) -> Result<T, E>
where
    T: Resource + Clone + Debug + DeserializeOwned + Serialize,
    S1: ToString,
    S2: AsRef<str>,
    C: FnOnce(ObjectMeta) -> T,
    F: FnOnce(T) -> Result<T, E>,
    Eq: FnOnce(&T, &T) -> bool,
    E: From<Error>,
{
    match api.get(name.as_ref()).await {
        Err(Error::Api(ae)) if ae.code == 404 => {
            log::debug!("CreateOrUpdate - Err(Api(404))");
            let object: T = creator(ObjectMeta {
                namespace: namespace.map(|s| s.to_string()),
                name: Some(name.as_ref().to_string()),
                ..Default::default()
            });
            let object = mutator(object)?;
            api.create(&PostParams::default(), &object).await?;
            Ok(object)
        }
        Err(e) => {
            log::info!("Error - {}", e);
            Err(e)?
        }
        Ok(object) => {
            log::debug!("CreateOrUpdate - Ok(...)");
            let new_object = mutator(object.clone())?;

            // only update when necessary
            if !eq(&object, &new_object) {
                log::debug!("CreateOrUpdate - Changed -> replacing");
                api.replace(name.as_ref(), &PostParams::default(), &new_object)
                    .await?;
            }
            Ok(new_object)
        }
    }
}

/// Create or update a Kubernetes resource.
pub async fn create_or_update<T, S1, S2, F, E>(
    api: &Api<T>,
    namespace: Option<S1>,
    name: S2,
    mutator: F,
) -> Result<T, E>
where
    T: Resource + Clone + Debug + DeserializeOwned + Serialize + PartialEq + Default,
    S1: ToString,
    S2: AsRef<str>,
    F: FnOnce(T) -> Result<T, E>,
    E: From<Error>,
{
    create_or_update_by(
        api,
        namespace,
        name,
        |meta| {
            let mut object: T = Default::default();
            *object.meta_mut() = meta;
            object
        },
        |this, that| this == that,
        mutator,
    )
    .await
}
