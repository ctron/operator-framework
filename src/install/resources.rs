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

use k8s_openapi::api::core::v1::{Container, ResourceRequirements};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;

pub trait SetResources {
    fn set_resources<S1, S2, S3>(
        &mut self,
        resource_type: S1,
        request: Option<S2>,
        limit: Option<S3>,
    ) where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>;
}

impl SetResources for ResourceRequirements {
    fn set_resources<S1, S2, S3>(
        &mut self,
        resource_type: S1,
        request: Option<S2>,
        limit: Option<S3>,
    ) where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        let resource_type = resource_type.into();
        match request {
            Some(request) => self.requests.use_or_create(|requests| {
                requests.insert(resource_type.clone(), Quantity(request.into()));
            }),
            None => {
                if let Some(requests) = &mut self.requests {
                    requests.remove(&resource_type);
                }
            }
        };
        match limit {
            Some(limit) => self.limits.use_or_create(|limits| {
                limits.insert(resource_type, Quantity(limit.into()));
            }),
            None => {
                if let Some(limits) = &mut self.limits {
                    limits.remove(&resource_type);
                }
            }
        };
    }
}

impl SetResources for Container {
    fn set_resources<S1, S2, S3>(
        &mut self,
        resource_type: S1,
        request: Option<S2>,
        limit: Option<S3>,
    ) where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        self.resources.use_or_create(|resources| {
            resources.set_resources(resource_type, request, limit);
        });
    }
}
