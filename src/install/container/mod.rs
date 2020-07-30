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
mod resources;

pub use self::args::*;
pub use self::env::*;
pub use self::port::*;
pub use self::resources::*;

use crate::utils::UseOrCreate;

use anyhow::Result;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Container, PodTemplateSpec};

pub trait ApplyContainer {
    fn apply_container<F>(&mut self, name: &str, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut Container) -> Result<()>;
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
