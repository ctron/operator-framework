use k8s_openapi::api::core::v1::Container;

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
pub trait SetArgs<S: ToString> {
    fn args(&mut self, args: Vec<S>);
}

impl<S: ToString> SetArgs<S> for Container {
    fn args(&mut self, args: Vec<S>) {
        self.args = Some(args.iter().map(|s| s.to_string()).collect());
    }
}
