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
use anyhow::Result;

/// Use the value of something optional, or create it first.
pub trait UseOrCreate<T> {
    fn use_or_create<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R;

    fn use_or_create_err<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut T) -> Result<()>,
    {
        self.use_or_create(|value| f(value))
    }
}

/// Implementation for `Option`s which wrap `Default`s.
impl<T> UseOrCreate<T> for Option<T>
where
    T: Default,
{
    fn use_or_create<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        match self {
            Some(value) => f(value),
            None => {
                let mut value = Default::default();
                let result = f(&mut value);
                self.replace(value);
                result
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Default, Debug)]
    struct Example {
        foo: String,
    }

    #[test]
    fn test_with_none() {
        let mut v: Option<Example> = None;
        v.use_or_create(|v| {
            v.foo = "bar".to_string();
        });

        assert!(v.is_some());
        assert_eq!(v.unwrap().foo, "bar");
    }

    #[test]
    fn test_with_some() {
        let mut v: Option<Example> = Some(Example { foo: "foo".into() });
        v.use_or_create(|v| {
            assert_eq!(v.foo, "foo");
            v.foo = "bar".to_string();
        });

        assert!(v.is_some());
        assert_eq!(v.unwrap().foo, "bar");
    }
}
