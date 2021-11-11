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

use crate::utils::UseOrCreate;
use anyhow::{anyhow, Error};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use std::borrow::Cow;

pub trait Meta {
    fn metadata(&self) -> &ObjectMeta;
    fn metadata_mut(&mut self) -> &mut ObjectMeta;

    fn kind(&self) -> Cow<'_, str>;
    fn api_version(&self) -> Cow<'_, str>;
}

pub trait OwnedBy<R> {
    fn owned_by(
        &mut self,
        resource: &R,
        controller: bool,
        block_owner_deletion: Option<bool>,
    ) -> Result<(), anyhow::Error>;

    fn owned_by_controller(&mut self, resource: &R) -> Result<(), anyhow::Error> {
        self.owned_by(resource, true, None)
    }

    fn is_owned_by(&self, owner: &R, controller: Option<bool>) -> Result<bool, anyhow::Error>;

    fn is_owned_by_controller(&self, owner: &R) -> Result<bool, anyhow::Error> {
        self.is_owned_by(owner, Some(true))
    }
}

pub trait SameOwner {
    fn is_same_owner(&self, other: &OwnerReference) -> bool {
        self.is_same_owner_opts(other, false)
    }

    fn is_same_owner_opts(&self, other: &OwnerReference, check_controller: bool) -> bool;
}

impl SameOwner for OwnerReference {
    fn is_same_owner_opts(&self, other: &OwnerReference, check_controller: bool) -> bool {
        if check_controller {
            // we check the controller first
            let self_controller = self.controller.unwrap_or(false);
            let other_controller = other.controller.unwrap_or(false);
            // if the controller flags don't match
            if self_controller != other_controller {
                // we can abort early
                return false;
            }
        }

        return self.kind == other.kind
            && self.api_version == other.api_version
            && self.name == other.name;
    }
}

pub trait AsOwner {
    fn as_owner(
        &self,
        controller: Option<bool>,
        block_owner_deletion: Option<bool>,
    ) -> Result<OwnerReference, anyhow::Error>;

    fn as_controller_owner(&self) -> Result<OwnerReference, anyhow::Error> {
        self.as_owner(Some(true), None)
    }
}

impl<K> AsOwner for K
where
    K: Meta,
{
    fn as_owner(
        &self,
        controller: Option<bool>,
        block_owner_deletion: Option<bool>,
    ) -> Result<OwnerReference, Error> {
        let name = self
            .metadata()
            .name
            .as_ref()
            .ok_or_else(|| anyhow!("Missing name"))?
            .clone();
        let uid = self
            .metadata()
            .uid
            .as_ref()
            .ok_or_else(|| anyhow!("Missing UID"))?
            .clone();

        Ok(OwnerReference {
            kind: self.kind().to_string(),
            api_version: self.api_version().to_string(),
            name,
            uid,
            controller,
            block_owner_deletion,
        })
    }
}

impl<K> Meta for K
where
    K: kube::Resource<DynamicType = ()>,
{
    fn metadata(&self) -> &ObjectMeta {
        self.meta()
    }

    fn metadata_mut(&mut self) -> &mut ObjectMeta {
        self.meta_mut()
    }

    fn kind(&self) -> Cow<'_, str> {
        Self::kind(&())
    }

    fn api_version(&self) -> Cow<'_, str> {
        Self::api_version(&())
    }
}

impl<K, R> OwnedBy<R> for K
where
    K: Meta,
    R: Meta,
{
    fn owned_by(
        &mut self,
        resource: &R,
        controller: bool,
        block_owner_deletion: Option<bool>,
    ) -> Result<(), anyhow::Error> {
        match (&self.metadata().namespace, &resource.metadata().namespace) {
            (None, None) => Ok(()),
            (Some(_), None) => Ok(()),

            (Some(obj_ns), Some(owner_ns)) => {
                if obj_ns == owner_ns {
                    Ok(())
                } else {
                    Err(anyhow!(
                        "If both objects are namespaced, they must belong to the same namespace"
                    ))
                }
            }
            (None, Some(_)) => Err(anyhow!(
                "Cluster scoped object must not have a namespaced owner"
            )),
        }?;

        let owner = resource.as_owner(Some(controller), block_owner_deletion)?;

        let mut found = None;

        self.metadata_mut()
            .owner_references
            .use_or_create_err(|owners| {
                for (idx, o) in owners.iter().enumerate() {
                    if owner.is_same_owner(&o) {
                        found = Some(idx);
                    } else if controller {
                        match o.controller {
                            Some(true) => Err(anyhow!("Object already has a controller")),
                            _ => Ok(()),
                        }?;
                    }
                }

                match found {
                    Some(idx) => {
                        let o = &mut owners[idx];
                        o.controller = owner.controller;
                        o.block_owner_deletion = owner.block_owner_deletion;
                    }
                    None => {
                        owners.push(owner);
                    }
                }

                Ok(())
            })?;

        Ok(())
    }

    fn is_owned_by(&self, owner: &R, controlled: Option<bool>) -> Result<bool, anyhow::Error> {
        let owner = owner.as_owner(controlled, None)?;

        if let Some(owner_refs) = &self.metadata().owner_references {
            for r in owner_refs {
                if r.is_same_owner_opts(&owner, controlled.is_some()) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use k8s_openapi::api::core::v1::ConfigMap;

    /// Create a new config map for testing
    fn new_cm(namespace: Option<&str>, name: &str, uid: &str) -> ConfigMap {
        ConfigMap {
            metadata: ObjectMeta {
                name: Some(name.into()),
                namespace: namespace.map(|s| s.to_string()),
                uid: Some(uid.to_string()),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_owned_by() {
        let mut config_map_1: ConfigMap = new_cm(Some("ns1"), "cm1", "123");
        let config_map_2: ConfigMap = new_cm(Some("ns1"), "cm2", "456");

        let r = config_map_1.owned_by(&config_map_2, false, None);
        assert!(r.is_ok(), "Should be ok");
        assert_eq!(
            1,
            config_map_1
                .metadata
                .owner_references
                .unwrap_or_default()
                .len()
        )
    }

    #[test]
    fn test_owned_by_multiple() {
        let mut config_map_1: ConfigMap = new_cm(Some("ns1"), "cm1", "123");
        let config_map_2: ConfigMap = new_cm(Some("ns1"), "cm2", "456");
        let config_map_3: ConfigMap = new_cm(Some("ns1"), "cm3", "789");

        let r = config_map_1.owned_by(&config_map_2, false, None);
        assert!(r.is_ok(), "Should be ok");
        let r = config_map_1.owned_by(&config_map_3, false, None);
        assert!(r.is_ok(), "Should be ok");
        assert_eq!(
            2,
            config_map_1
                .metadata
                .owner_references
                .unwrap_or_default()
                .len()
        )
    }

    #[test]
    fn test_owned_by_multiple_replace() {
        let mut config_map_1: ConfigMap = new_cm(Some("ns1"), "cm1", "123");
        let config_map_2: ConfigMap = new_cm(Some("ns1"), "cm2", "456");
        let config_map_3: ConfigMap = new_cm(Some("ns1"), "cm3", "789");

        let r = config_map_1.owned_by(&config_map_2, false, None);
        assert!(r.is_ok(), "Should be ok");
        let r = config_map_1.owned_by(&config_map_3, false, None);
        assert!(r.is_ok(), "Should be ok");

        let config_map_4: ConfigMap = new_cm(Some("ns1"), "cm3", "AAA");
        let r = config_map_1.owned_by(&config_map_4, false, None);
        assert!(r.is_ok(), "Should be ok");
        assert_eq!(
            2,
            config_map_1
                .metadata
                .owner_references
                .unwrap_or_default()
                .len()
        )
    }

    #[test]
    fn test_owned_by_controller() {
        let mut config_map_1: ConfigMap = new_cm(Some("ns1"), "cm1", "123");
        let config_map_2: ConfigMap = new_cm(Some("ns1"), "cm2", "456");

        let r = config_map_1.owned_by_controller(&config_map_2);
        assert!(r.is_ok(), "Should be ok");
        assert_eq!(
            1,
            config_map_1
                .metadata
                .owner_references
                .unwrap_or_default()
                .len()
        )
    }

    #[test]
    fn test_owned_by_controller_only_one() {
        let mut config_map_1: ConfigMap = new_cm(Some("ns1"), "cm1", "123");
        let config_map_2: ConfigMap = new_cm(Some("ns1"), "cm2", "456");
        let config_map_3: ConfigMap = new_cm(Some("ns1"), "cm3", "789");
        let config_map_4: ConfigMap = new_cm(Some("ns1"), "cm4", "012");

        let r = config_map_1.owned_by_controller(&config_map_2);
        assert!(r.is_ok(), "Should be ok");
        let r = config_map_1.owned_by(&config_map_3, false, None);
        assert!(r.is_ok(), "Should be ok");
        let r = config_map_1.owned_by_controller(&config_map_4);
        assert!(r.is_err(), "Must not be ok");
        assert_eq!(
            2,
            config_map_1
                .metadata
                .owner_references
                .unwrap_or_default()
                .len()
        )
    }

    #[test]
    fn test_is_owned_by() {
        let mut config_map_1: ConfigMap = new_cm(Some("ns1"), "cm1", "123");
        let config_map_2: ConfigMap = new_cm(Some("ns1"), "cm2", "456");

        let r = config_map_1.owned_by(&config_map_2, false, None);
        assert!(r.is_ok());

        assert_eq!(
            true,
            config_map_1
                .is_owned_by(&config_map_2, Some(false))
                .unwrap()
        );
        assert_eq!(
            false,
            config_map_2
                .is_owned_by(&config_map_1, Some(false))
                .unwrap()
        );

        assert_eq!(true, config_map_1.is_owned_by(&config_map_2, None).unwrap());
        assert_eq!(
            false,
            config_map_2.is_owned_by(&config_map_1, None).unwrap()
        );
    }

    #[test]
    fn test_is_controlled_by() {
        let mut config_map_1: ConfigMap = new_cm(Some("ns1"), "cm1", "123");
        let config_map_2: ConfigMap = new_cm(Some("ns1"), "cm2", "456");
        let config_map_3: ConfigMap = new_cm(Some("ns1"), "cm3", "789");

        config_map_1.owned_by_controller(&config_map_2).unwrap();
        config_map_1.owned_by(&config_map_3, false, None).unwrap();

        assert_eq!(
            true,
            config_map_1.is_owned_by_controller(&config_map_2).unwrap()
        );
        assert_eq!(
            false,
            config_map_1.is_owned_by_controller(&config_map_3).unwrap()
        );

        assert_eq!(true, config_map_1.is_owned_by(&config_map_2, None).unwrap());
        assert_eq!(true, config_map_1.is_owned_by(&config_map_3, None).unwrap());
    }
}
