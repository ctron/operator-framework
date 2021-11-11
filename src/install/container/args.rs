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

use k8s_openapi::api::core::v1::Container;

pub trait SetArgs<T, I>
where
    T: Into<String>,
    I: IntoIterator<Item = T>,
{
    fn args(&mut self, args: I);
}

impl<T, I> SetArgs<T, I> for Container
where
    T: Into<String>,
    I: IntoIterator<Item = T>,
{
    fn args(&mut self, args: I) {
        self.args = Some(args.into_iter().map(|s| s.into()).collect());
    }
}

pub trait SetCommand<T, I>
where
    T: Into<String>,
    I: IntoIterator<Item = T>,
{
    fn command(&mut self, args: I);
}

impl<T, I> SetCommand<T, I> for Container
where
    T: Into<String>,
    I: IntoIterator<Item = T>,
{
    fn command(&mut self, command: I) {
        self.command = Some(command.into_iter().map(|s| s.into()).collect());
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test() {
        let mut container = Container {
            ..Default::default()
        };
        container.command(vec!["foo", "bar"]);
        container.args(vec!["foo", "bar"]);
    }
}
