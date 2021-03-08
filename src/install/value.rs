use crate::install::container::ApplyEnvironmentVariable;
use anyhow::Result;
use async_trait::async_trait;
use core::fmt::{self, Formatter};
use k8s_openapi::api::core::v1::{
    ConfigMap, ConfigMapKeySelector, EnvVar, EnvVarSource, Secret, SecretKeySelector,
};
use kube::{api::Meta, Api};
use serde::de::DeserializeOwned;
use serde::{
    de::{self, MapAccess, Visitor},
    ser::SerializeStruct,
    {Deserialize, Deserializer, Serialize, Serializer},
};

#[derive(Debug, Clone, PartialEq)]
pub enum ValueOrReference {
    Value(String),
    Secret(SecretKeySelector),
    ConfigMap(ConfigMapKeySelector),
}

#[async_trait]
pub trait Reader {
    /// Read a value from a configmap. Only returns `None` if the selector was optional.
    async fn read_configmap(&self, selector: &ConfigMapKeySelector) -> Result<Option<String>>;
    /// Read a value from a secret. Only returns `None` if the selector was optional.
    async fn read_secret(&self, selector: &SecretKeySelector) -> Result<Option<String>>;
}

pub struct KubeReader<'a> {
    configmaps: &'a Api<ConfigMap>,
    secrets: &'a Api<Secret>,
}

impl<'a> KubeReader<'a> {
    pub fn new(configmaps: &'a Api<ConfigMap>, secrets: &'a Api<Secret>) -> Self {
        Self {
            configmaps,
            secrets,
        }
    }

    fn no_result(optional: bool, ty: &str, name: &str, key: &str) -> Result<Option<String>> {
        if optional {
            Ok(None)
        } else {
            anyhow::bail!("Missing key '{}' in {} '{}'", key, ty, name)
        }
    }

    async fn read<T, F>(
        ty: &str,
        api: &Api<T>,
        name: Option<&str>,
        key: &str,
        optional: Option<bool>,
        extractor: F,
    ) -> Result<Option<String>>
    where
        T: Meta + DeserializeOwned + Clone,
        F: FnOnce(T, &str) -> Option<String>,
    {
        if let Some(name) = name {
            let optional = optional.unwrap_or_default();

            match api.get(&name).await {
                Ok(resource) => match extractor(resource, key) {
                    Some(value) => Ok(Some(value)),
                    None => Self::no_result(optional, ty, name, key),
                },
                Err(kube::Error::Api(err)) if err.reason == "NotFound" => {
                    Self::no_result(optional, ty, name, key)
                }
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl<'a> Reader for KubeReader<'a> {
    async fn read_configmap(&self, selector: &ConfigMapKeySelector) -> Result<Option<String>> {
        Self::read(
            "ConfigMap",
            &self.configmaps,
            selector.name.as_ref().map(|s| s.as_str()),
            &selector.key,
            selector.optional,
            |resource, key| resource.data.and_then(|data| data.get(key).cloned()),
        )
        .await
    }

    async fn read_secret(&self, selector: &SecretKeySelector) -> Result<Option<String>> {
        Self::read(
            "Secret",
            &self.secrets,
            selector.name.as_ref().map(|s| s.as_str()),
            &selector.key,
            selector.optional,
            |resource, key| {
                resource.data.and_then(|data| {
                    data.get(key)
                        .cloned()
                        .and_then(|s| String::from_utf8(s.0).ok())
                })
            },
        )
        .await
    }
}

impl ValueOrReference {
    /// apply the value (or reference) to an env-var
    pub fn apply_to_envvar(&self, env: &mut EnvVar) {
        match self {
            Self::Value(value) => {
                env.value = Some(value.into());
                env.value_from = None;
            }
            Self::ConfigMap(selector) => {
                env.value = None;
                env.value_from = Some(EnvVarSource {
                    config_map_key_ref: Some(selector.clone()),
                    field_ref: None,
                    resource_field_ref: None,
                    secret_key_ref: None,
                });
            }
            Self::Secret(selector) => {
                env.value = None;
                env.value_from = Some(EnvVarSource {
                    config_map_key_ref: None,
                    field_ref: None,
                    resource_field_ref: None,
                    secret_key_ref: Some(selector.clone()),
                });
            }
        }
    }

    /// Apply the value as an environment variable to a ['ApplyEnvironmentVariable'], e.g. a ['Container'].
    pub fn apply_to_env<E, S>(&self, env: &mut E, name: S)
    where
        E: ApplyEnvironmentVariable,
        S: AsRef<str>,
    {
        env.apply_env(name, |envvar| {
            self.apply_to_envvar(envvar);
            Ok(())
        })
        // we can unwrap here as we are not returning an error in our mutator
        .unwrap();
    }

    /// Read the actual value.
    ///
    /// This may either return the value directly, or do a remote call to read the value.
    pub async fn read_value<R>(&self, reader: &R) -> Result<Option<String>>
    where
        R: Reader,
    {
        match self {
            Self::Value(value) => Ok(Some(value.clone())),
            Self::ConfigMap(selector) => reader.read_configmap(selector).await,
            Self::Secret(selector) => reader.read_secret(selector).await,
        }
    }
}

impl<'de> Deserialize<'de> for ValueOrReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;
        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = ValueOrReference;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("string or map")
            }

            fn visit_str<E>(self, value: &str) -> Result<ValueOrReference, E> {
                Ok(ValueOrReference::Value(value.to_string()))
            }

            fn visit_map<V>(self, mut map: V) -> Result<ValueOrReference, V::Error>
            where
                V: MapAccess<'de>,
            {
                if let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "configMap" => Ok(ValueOrReference::ConfigMap(map.next_value()?)),
                        "secret" => Ok(ValueOrReference::Secret(map.next_value()?)),
                        t => Err(de::Error::unknown_variant(t, &["configMap", "secret"])),
                    }
                } else {
                    Err(de::Error::custom("No value type present"))
                }
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Serialize for ValueOrReference {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Value(value) => serializer.serialize_str(value),
            Self::ConfigMap(selector) => {
                let mut s = serializer.serialize_struct("configMap", 1)?;
                s.serialize_field("configMap", selector)?;
                s.end()
            }
            Self::Secret(selector) => {
                let mut s = serializer.serialize_struct("secret", 1)?;
                s.serialize_field("secret", selector)?;
                s.end()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use serde_json::{json, Value};

    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct MyCrd {
        pub field_one: ValueOrReference,
    }

    fn test_combination(crd: MyCrd, value: Value) -> Result<()> {
        let enc = serde_json::to_value(&crd)?;

        println!("Encoded: {}", enc);
        // test encoding of crd
        assert_eq!(enc, value);
        // test decoding of crd
        assert_eq!(crd, serde_json::from_value(value)?);

        Ok(())
    }

    #[test]
    fn test_value() -> Result<()> {
        test_combination(
            MyCrd {
                field_one: ValueOrReference::Value("foo".to_string()),
            },
            json!({
                "fieldOne": "foo",
            }),
        )?;

        Ok(())
    }

    #[test]
    fn test_configmap() -> Result<()> {
        test_combination(
            MyCrd {
                field_one: ValueOrReference::ConfigMap(ConfigMapKeySelector {
                    name: Some("foo".to_string()),
                    key: "bar".to_string(),
                    ..Default::default()
                }),
            },
            json!({
                "fieldOne": {
                    "configMap": {
                        "name": "foo",
                        "key": "bar",
                    }
                }
            }),
        )?;

        Ok(())
    }

    #[test]
    fn test_secret() -> Result<()> {
        test_combination(
            MyCrd {
                field_one: ValueOrReference::Secret(SecretKeySelector {
                    name: Some("foo".to_string()),
                    key: "bar".to_string(),
                    ..Default::default()
                }),
            },
            json!({
                "fieldOne": {
                    "secret": {
                        "name": "foo",
                        "key": "bar",
                    }
                }
            }),
        )?;

        Ok(())
    }

    #[test]
    fn test_wrong_type() -> Result<()> {
        let crd: serde_json::Result<MyCrd> = serde_json::from_value(json!({"fieldOne": {
            "foo": "bar",
        }}));

        assert!(crd.is_err());

        Ok(())
    }
}
