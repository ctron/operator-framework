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

use k8s_openapi::api::core::v1::{Container, PodSpec, Volume, VolumeMount};

pub trait ApplyVolume {
    fn apply_volume<F, S>(&mut self, name: S, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut Volume) -> Result<()>,
        S: AsRef<str>;
}

impl ApplyVolume for Vec<Volume> {
    fn apply_volume<F, S>(&mut self, name: S, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut Volume) -> Result<()>,
        S: AsRef<str>,
    {
        let c = self.iter_mut().find(|c| &c.name == name.as_ref());
        match c {
            Some(c) => {
                mutator(c)?;
            }
            None => {
                let mut item: Volume = Default::default();
                item.name = name.as_ref().to_string();
                mutator(&mut item)?;
                self.push(item);
            }
        }
        Ok(())
    }
}

impl ApplyVolume for PodSpec {
    fn apply_volume<F, S>(&mut self, name: S, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut Volume) -> Result<()>,
        S: AsRef<str>,
    {
        self.volumes
            .use_or_create(|volumes| volumes.apply_volume(name, mutator))
    }
}

pub trait ApplyVolumeMount {
    fn apply_volume_mount<F, S>(&mut self, name: S, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut VolumeMount) -> Result<()>,
        S: AsRef<str>;
}

impl ApplyVolumeMount for Vec<VolumeMount> {
    fn apply_volume_mount<F, S>(&mut self, name: S, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut VolumeMount) -> Result<()>,
        S: AsRef<str>,
    {
        let c = self.iter_mut().find(|c| &c.name == name.as_ref());
        match c {
            Some(c) => {
                mutator(c)?;
            }
            None => {
                let mut item: VolumeMount = Default::default();
                item.name = name.as_ref().to_string();
                mutator(&mut item)?;
                self.push(item);
            }
        }
        Ok(())
    }
}

impl ApplyVolumeMount for Container {
    fn apply_volume_mount<F, S>(&mut self, name: S, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut VolumeMount) -> Result<()>,
        S: AsRef<str>,
    {
        self.volume_mounts
            .use_or_create(|volume_mounts| volume_mounts.apply_volume_mount(name, mutator))
    }
}
