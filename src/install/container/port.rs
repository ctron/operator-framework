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
use anyhow::Result;
use k8s_openapi::api::core::v1::{Container, ContainerPort};

pub trait ApplyPort {
    fn apply_port<F, S>(&mut self, name: S, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut ContainerPort) -> Result<()>,
        S: AsRef<str>;

    fn add_port<S>(&mut self, name: S, container_port: i32, protocol: Option<String>) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.apply_port(name, |c| {
            c.container_port = container_port;
            c.protocol = protocol;
            Ok(())
        })
    }
}

impl ApplyPort for Vec<ContainerPort> {
    fn apply_port<F, S>(&mut self, name: S, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut ContainerPort) -> Result<()>,
        S: AsRef<str>,
    {
        let c = self.iter_mut().find(|c| match &c.name {
            None => false,
            Some(s) => s.as_str() == name.as_ref(),
        });
        match c {
            Some(c) => {
                mutator(c)?;
            }
            None => {
                let mut port: ContainerPort = Default::default();
                port.name = Some(name.as_ref().to_string());
                mutator(&mut port)?;
                self.push(port);
            }
        }
        Ok(())
    }
}

impl ApplyPort for Container {
    fn apply_port<F, S>(&mut self, name: S, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut ContainerPort) -> Result<()>,
        S: AsRef<str>,
    {
        self.ports
            .use_or_create(|ports| ports.apply_port(name, mutator))
    }
}
