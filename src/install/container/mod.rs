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
mod args;
mod env;
mod port;
mod volumes;

pub use self::args::*;
pub use self::env::*;
pub use self::port::*;
pub use self::volumes::*;
pub use crate::install::resources::*;

use crate::utils::UseOrCreate;

use anyhow::Result;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Container, PodTemplateSpec};

pub trait ApplyContainer {
    fn apply_container<F>(&mut self, name: &str, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut Container) -> Result<()>;
}

pub trait RemoveContainer {
    /// removes all containers matching the predicate
    fn remove_containers<F>(&mut self, predicate: F) -> usize
    where
        F: Fn(&Container) -> bool;

    /// remove a container by name
    fn remove_container_by_name<S: AsRef<str>>(&mut self, name: S) -> bool {
        self.remove_containers(|c| c.name == name.as_ref()) > 0
    }
}

impl ApplyContainer for Vec<Container> {
    fn apply_container<F>(&mut self, name: &str, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut Container) -> Result<()>,
    {
        let c = self.iter_mut().find(|c| c.name == name);
        match c {
            Some(c) => {
                mutator(c)?;
            }
            None => {
                let mut container: Container = Default::default();
                container.name = name.into();
                mutator(&mut container)?;
                self.push(container);
            }
        }

        Ok(())
    }
}

impl ApplyContainer for Option<Vec<Container>> {
    fn apply_container<F>(&mut self, name: &str, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut Container) -> Result<()>,
    {
        self.use_or_create(|containers| containers.apply_container(name, mutator))
    }
}

impl ApplyContainer for PodTemplateSpec {
    fn apply_container<F>(&mut self, name: &str, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut Container) -> Result<()>,
    {
        self.spec
            .use_or_create(|spec| spec.containers.apply_container(name, mutator))
    }
}

impl ApplyContainer for Deployment {
    fn apply_container<F>(&mut self, name: &str, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut Container) -> Result<()>,
    {
        self.spec
            .use_or_create(|spec| spec.template.apply_container(name, mutator))
    }
}

impl RemoveContainer for Vec<Container> {
    fn remove_containers<F>(&mut self, predicate: F) -> usize
    where
        F: Fn(&Container) -> bool,
    {
        let mut n: usize = 0;
        self.retain(|c| {
            if predicate(c) {
                n += 1;
                false
            } else {
                true
            }
        });
        n
    }
}

impl RemoveContainer for Option<&mut Vec<Container>> {
    fn remove_containers<F>(&mut self, predicate: F) -> usize
    where
        F: Fn(&Container) -> bool,
    {
        if let Some(containers) = self {
            containers.remove_containers(predicate)
        } else {
            0
        }
    }
}

impl RemoveContainer for PodTemplateSpec {
    fn remove_containers<F>(&mut self, predicate: F) -> usize
    where
        F: Fn(&Container) -> bool,
    {
        self.spec
            .as_mut()
            .map(|s| &mut s.containers)
            .remove_containers(predicate)
    }
}

impl RemoveContainer for Deployment {
    fn remove_containers<F>(&mut self, predicate: F) -> usize
    where
        F: Fn(&Container) -> bool,
    {
        self.spec
            .as_mut()
            .map(|s| &mut s.template)
            .and_then(|s| s.spec.as_mut())
            .map(|s| &mut s.containers)
            .remove_containers(predicate)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use k8s_openapi::api::apps::v1::Deployment;

    /// test apply on different targets
    #[test]
    fn test_apply() {
        let mut d = Deployment::default();
        d.apply_container("foo", |_| Ok(())).unwrap();

        fn test(dm: &mut Deployment) {
            dm.apply_container("foo", |_| Ok(())).unwrap();
        }

        test(&mut d);
    }
}
