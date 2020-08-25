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

pub trait DropVolume {
    fn drop_volume<S>(&mut self, name: S) -> bool
    where
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

impl DropVolume for Vec<Volume> {
    fn drop_volume<S>(&mut self, name: S) -> bool
    where
        S: AsRef<str>,
    {
        let start = self.len();
        self.retain(|v| v.name != name.as_ref());

        start != self.len()
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

impl DropVolume for PodSpec {
    fn drop_volume<S>(&mut self, name: S) -> bool
    where
        S: AsRef<str>,
    {
        if let Some(v) = &mut self.volumes {
            v.drop_volume(name)
        } else {
            false
        }
    }
}

pub trait ApplyVolumeMount {
    fn apply_volume_mount<F, S>(&mut self, name: S, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut VolumeMount) -> Result<()>,
        S: AsRef<str>;

    fn apply_volume_mount_simple<S1, S2>(
        &mut self,
        name: S1,
        path: S2,
        read_only: bool,
    ) -> Result<()>
    where
        S1: AsRef<str>,
        S2: ToString,
    {
        self.apply_volume_mount(name, |mount| {
            mount.mount_path = path.to_string();
            mount.read_only = Some(read_only);
            mount.mount_propagation = None;
            mount.sub_path = None;
            mount.sub_path_expr = None;
            Ok(())
        })
    }
}

pub trait DropVolumeMount {
    fn drop_volume_mount<S>(&mut self, name: S) -> bool
    where
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

impl DropVolumeMount for Vec<VolumeMount> {
    fn drop_volume_mount<S>(&mut self, name: S) -> bool
    where
        S: AsRef<str>,
    {
        let start = self.len();
        self.retain(|v| v.name != name.as_ref());

        start != self.len()
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

impl DropVolumeMount for Container {
    fn drop_volume_mount<S>(&mut self, name: S) -> bool
    where
        S: AsRef<str>,
    {
        if let Some(v) = &mut self.volume_mounts {
            v.drop_volume_mount(name)
        } else {
            false
        }
    }
}
