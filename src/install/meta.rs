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

use anyhow::anyhow;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use k8s_openapi::Metadata;

use crate::utils::UseOrCreate;

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
}

pub trait SameOwner {
    fn is_same_owner(&self, other: &OwnerReference) -> bool;
}

impl SameOwner for OwnerReference {
    fn is_same_owner(&self, other: &OwnerReference) -> bool {
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
    K: Metadata<Ty = ObjectMeta>,
{
    fn as_owner(
        &self,
        controller: Option<bool>,
        block_owner_deletion: Option<bool>,
    ) -> Result<OwnerReference, anyhow::Error> {
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
            kind: Self::KIND.to_string(),
            api_version: Self::API_VERSION.to_string(),
            name,
            uid,
            controller,
            block_owner_deletion,
        })
    }
}

impl<K, R> OwnedBy<R> for K
where
    K: Metadata<Ty = ObjectMeta>,
    R: Metadata<Ty = ObjectMeta>,
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

        self.metadata_mut()
            .owner_references
            .use_or_create_err(|owners| {
                let mut found = None;

                for (idx, o) in owners.iter().enumerate() {
                    if owner.is_same_owner(&o) {
                        found = Some(idx);
                    } else {
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
        assert_eq!(1, config_map_1.metadata.owner_references.expect("").len())
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
        assert_eq!(2, config_map_1.metadata.owner_references.expect("").len())
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
        assert_eq!(2, config_map_1.metadata.owner_references.expect("").len())
    }

    #[test]
    fn test_owned_by_controller() {
        let mut config_map_1: ConfigMap = new_cm(Some("ns1"), "cm1", "123");
        let config_map_2: ConfigMap = new_cm(Some("ns1"), "cm2", "456");

        let r = config_map_1.owned_by_controller(&config_map_2);
        assert!(r.is_ok(), "Should be ok");
        assert_eq!(1, config_map_1.metadata.owner_references.expect("").len())
    }

    #[test]
    fn test_owned_by_controller_only_one() {
        let mut config_map_1: ConfigMap = new_cm(Some("ns1"), "cm1", "123");
        let config_map_2: ConfigMap = new_cm(Some("ns1"), "cm2", "456");
        let config_map_3: ConfigMap = new_cm(Some("ns1"), "cm3", "789");

        let r = config_map_1.owned_by_controller(&config_map_2);
        assert!(r.is_ok(), "Should be ok");
        let r = config_map_1.owned_by_controller(&config_map_3);
        assert!(r.is_err(), "Must not be ok");
        assert_eq!(1, config_map_1.metadata.owner_references.expect("").len())
    }
}