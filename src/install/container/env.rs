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
use k8s_openapi::api::core::v1::{
    ConfigMapKeySelector, Container, EnvVar, EnvVarSource, ObjectFieldSelector,
    ResourceFieldSelector, SecretKeySelector,
};

pub trait ApplyEnvironmentVariable {
    /// Apply the mutator function to the environment variable with the provided name.
    ///
    /// If there currently exists no environment variable with this name, a new entry is created.
    ///
    /// The function may only throw an error if the mutator threw an error.
    fn apply_env<F, S>(&mut self, name: S, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut EnvVar) -> Result<()>,
        S: AsRef<str>;

    /// Drop an environment variable with the provided name.
    ///
    /// If no entry with that name exists, this is a no-op.
    fn drop_env<S>(&mut self, name: S)
    where
        S: AsRef<str>;

    fn add_env<S1, S2>(&mut self, name: S1, value: S2) -> Result<()>
    where
        S1: AsRef<str>,
        S2: Into<String>,
    {
        self.apply_env(name, |env| {
            env.value = Some(value.into());
            env.value_from = None;
            Ok(())
        })
    }

    fn set_env<S1, S2>(&mut self, name: S1, value: Option<S2>) -> Result<()>
    where
        S1: AsRef<str>,
        S2: Into<String>,
    {
        match value {
            Some(value) => self.add_env(name, value),
            None => {
                self.drop_env(name);
                Ok(())
            }
        }
    }

    fn add_env_from<S>(&mut self, name: S, from: EnvVarSource) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.apply_env(name, |env| {
            env.value = None;
            env.value_from = Some(from);
            Ok(())
        })
    }

    fn set_env_from<S>(&mut self, name: S, from: Option<EnvVarSource>) -> Result<()>
    where
        S: AsRef<str>,
    {
        match from {
            Some(from) => self.add_env_from(name, from),
            None => {
                self.drop_env(name);
                Ok(())
            }
        }
    }

    fn add_env_from_field<S>(&mut self, name: S, selector: ObjectFieldSelector) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.add_env_from(
            name,
            EnvVarSource {
                field_ref: Some(selector),
                ..Default::default()
            },
        )
    }

    fn set_env_from_field<S>(&mut self, name: S, from: Option<ObjectFieldSelector>) -> Result<()>
    where
        S: AsRef<str>,
    {
        match from {
            Some(from) => self.add_env_from_field(name, from),
            None => {
                self.drop_env(name);
                Ok(())
            }
        }
    }

    fn add_env_from_field_path<S1, S2>(&mut self, name: S1, path: S2) -> Result<()>
    where
        S1: AsRef<str>,
        S2: Into<String>,
    {
        self.add_env_from_field(
            name,
            ObjectFieldSelector {
                api_version: None,
                field_path: path.into(),
            },
        )
    }

    fn set_env_from_field_path<S1, S2>(&mut self, name: S1, path: Option<S2>) -> Result<()>
    where
        S1: AsRef<str>,
        S2: Into<String>,
    {
        match path {
            Some(path) => self.add_env_from_field_path(name, path),
            None => {
                self.drop_env(name);
                Ok(())
            }
        }
    }

    fn add_env_from_secret_selector<S>(
        &mut self,
        name: S,
        selector: SecretKeySelector,
    ) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.add_env_from(
            name,
            EnvVarSource {
                secret_key_ref: Some(selector),
                ..Default::default()
            },
        )
    }

    fn set_env_from_secret_select<S>(
        &mut self,
        name: S,
        from: Option<SecretKeySelector>,
    ) -> Result<()>
    where
        S: AsRef<str>,
    {
        match from {
            Some(from) => self.add_env_from_secret_selector(name, from),
            None => {
                self.drop_env(name);
                Ok(())
            }
        }
    }

    fn add_env_from_secret<S1, S2, S3>(&mut self, name: S1, secret_name: S2, key: S3) -> Result<()>
    where
        S1: AsRef<str>,
        S2: ToString,
        S3: ToString,
    {
        self.add_env_from_secret_selector(
            name,
            SecretKeySelector {
                name: Some(secret_name.to_string()),
                key: key.to_string(),
                optional: None,
            },
        )
    }

    fn add_env_from_configmap_selector<S>(
        &mut self,
        name: S,
        selector: ConfigMapKeySelector,
    ) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.add_env_from(
            name,
            EnvVarSource {
                config_map_key_ref: Some(selector),
                ..Default::default()
            },
        )
    }

    fn set_env_from_configmap_selector<S>(
        &mut self,
        name: S,
        from: Option<ConfigMapKeySelector>,
    ) -> Result<()>
    where
        S: AsRef<str>,
    {
        match from {
            Some(from) => self.add_env_from_configmap_selector(name, from),
            None => {
                self.drop_env(name);
                Ok(())
            }
        }
    }

    fn add_env_from_resource<S>(&mut self, name: S, selector: ResourceFieldSelector) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.add_env_from(
            name,
            EnvVarSource {
                resource_field_ref: Some(selector),
                ..Default::default()
            },
        )
    }

    fn set_env_from_resource_selector<S>(
        &mut self,
        name: S,
        from: Option<ResourceFieldSelector>,
    ) -> Result<()>
    where
        S: AsRef<str>,
    {
        match from {
            Some(from) => self.add_env_from_resource(name, from),
            None => {
                self.drop_env(name);
                Ok(())
            }
        }
    }
}

impl ApplyEnvironmentVariable for Vec<EnvVar> {
    fn apply_env<F, S>(&mut self, name: S, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut EnvVar) -> Result<()>,
        S: AsRef<str>,
    {
        let c = self.iter_mut().find(|c| c.name == name.as_ref());
        match c {
            Some(c) => {
                mutator(c)?;
            }
            None => {
                let mut entry: EnvVar = Default::default();
                entry.name = name.as_ref().to_string();
                mutator(&mut entry)?;
                self.push(entry);
            }
        }
        Ok(())
    }

    fn drop_env<S>(&mut self, name: S)
    where
        S: AsRef<str>,
    {
        self.retain(|env| env.name != name.as_ref());
    }
}

impl ApplyEnvironmentVariable for Container {
    fn apply_env<F, S>(&mut self, name: S, mutator: F) -> Result<()>
    where
        F: FnOnce(&mut EnvVar) -> Result<()>,
        S: AsRef<str>,
    {
        self.env.use_or_create(|env| env.apply_env(name, mutator))
    }

    fn drop_env<S>(&mut self, name: S)
    where
        S: AsRef<str>,
    {
        if let Some(envs) = &mut self.env {
            envs.drop_env(name);
        }
    }
}
