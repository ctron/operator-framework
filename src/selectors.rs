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

use std::collections::BTreeMap;

pub trait ToSelector {
    /// Convert to a valid selector expression
    fn to_selector(&self) -> String;
}

impl<S1, S2> ToSelector for BTreeMap<S1, S2>
where
    S1: ToString,
    S2: AsRef<str>,
{
    /// For a map, we generate an "and" expression, consisting of all key/value pairs.
    fn to_selector(&self) -> String {
        self.iter()
            .map(|(k, v)| k.to_string() + "=" + v.as_ref())
            .collect::<Vec<String>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_empty() {
        let labels = BTreeMap::<String, String>::new();
        assert_eq!("", labels.to_selector());
    }

    #[test]
    fn test_single() {
        let mut labels = BTreeMap::new();
        labels.insert("foo", "bar");
        assert_eq!("foo=bar", labels.to_selector());
    }

    #[test]
    fn test_multiple() {
        let mut labels = BTreeMap::new();
        labels.insert("foo", "bar");
        labels.insert("bar", "baz");
        let sel = labels.to_selector();

        // the map doesn't provide an order, so we need to check for both variants
        assert!(sel == "foo=bar,bar=baz" || sel == "bar=baz,foo=bar");
    }
}
