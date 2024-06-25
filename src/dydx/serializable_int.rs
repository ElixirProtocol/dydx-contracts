use num_bigint::BigInt;
use schemars::schema::{InstanceType, Schema, SchemaObject};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct SerializableInt {
    pub i: BigInt,
}

impl SerializableInt {
    pub fn new(i: BigInt) -> Self {
        Self { i }
    }
}

impl Serialize for SerializableInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.i.to_string())
    }
}

impl<'de> Deserialize<'de> for SerializableInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SerializableIntVisitor;

        impl<'de> serde::de::Visitor<'de> for SerializableIntVisitor {
            type Value = SerializableInt;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a big integer")
            }

            fn visit_str<E>(self, value: &str) -> Result<SerializableInt, E>
            where
                E: serde::de::Error,
            {
                let big_int = value.parse::<BigInt>().map_err(E::custom)?;
                Ok(SerializableInt::new(big_int))
            }
        }

        deserializer.deserialize_str(SerializableIntVisitor)
    }
}

impl JsonSchema for SerializableInt {
    fn schema_name() -> String {
        "SerializableInt".to_string()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("bigint".to_string()),
            metadata: Some(Box::new(schemars::schema::Metadata {
                description: Some("A big integer serialized as a string.".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}
