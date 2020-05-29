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
use anyhow::Result;

use kube::api::{Meta, ObjectMeta, PostParams};
use kube::{Api, Error};

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Create or update a Kubernetes resource.
pub async fn create_or_update<T, S1, S2, F>(
    api: &Api<T>,
    namespace: Option<S1>,
    name: S2,
    mutator: F,
) -> Result<()>
where
    T: Clone + Serialize + DeserializeOwned + Meta<Ty = ObjectMeta> + Default + PartialEq,
    S1: ToString,
    S2: AsRef<str>,
    F: FnOnce(T) -> Result<T>,
{
    match api.get(name.as_ref()).await {
        Err(Error::Api(ae)) if ae.code == 404 => {
            log::debug!("CreateOrUpdate - Err(Api(404))");
            let mut object: T = Default::default();
            object.set_metadata(ObjectMeta {
                namespace: namespace.map(|s| s.to_string()),
                name: Some(name.as_ref().to_string()),
                ..Default::default()
            });
            let object = mutator(object)?;
            api.create(&PostParams::default(), &object).await?;
        }
        Err(e) => {
            log::info!("Error - {}", e);
            Err(e)?;
        }
        Ok(object) => {
            log::debug!("CreateOrUpdate - Ok(...)");
            let new_object = mutator(object.clone())?;

            // only update when necessary
            if !object.eq(&new_object) {
                log::debug!("CreateOrUpdate - Changed -> replacing");
                api.replace(name.as_ref(), &PostParams::default(), &new_object)
                    .await?;
            }
        }
    };

    Ok(())
}
