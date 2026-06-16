use serde::Deserialize;
use serde_inline::serde_inline;

struct RawVec3 {
    x: i32,
    y: i32,
    z: i32,
}

impl<'de> Deserialize<'de> for RawVec3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let parts = string.split(' ').collect::<Vec<_>>();
        if parts.len() != 3 {
            return Err(serde::de::Error::custom("Expected 3 components"));
        }
        let x = parts[0].parse::<i32>().map_err(serde::de::Error::custom)?;
        let y = parts[1].parse::<i32>().map_err(serde::de::Error::custom)?;
        let z = parts[2].parse::<i32>().map_err(serde::de::Error::custom)?;
        Ok(RawVec3 { x, y, z })
    }
}

struct Vec3(RawVec3);

impl From<RawVec3> for Vec3 {
    fn from(raw: RawVec3) -> Self {
        Vec3(raw)
    }
}

#[serde_inline]
#[derive(Deserialize)]
struct Example {
    #[serde_inline(
        deserialize_with = |deserializer| RawVec3::deserialize(deserializer).map(Into::into),
        default = Vec3(RawVec3 { x: 0, y: 0, z: 0 })
    )]
    position: Vec3,
}

#[serde_inline]
#[derive(Deserialize)]
struct TupleExample(
    #[serde_inline(deserialize_with = |deserializer| {
        RawVec3::deserialize(deserializer).map(Into::into)
    })]
    Vec3,
);

#[derive(Deserialize)]
struct TupleExampleWrapper {
    inner: TupleExample,
}

#[serde_inline]
#[derive(Deserialize)]
enum ExampleEnum {
    Named {
        #[serde_inline(deserialize_with = |deserializer| {
            RawVec3::deserialize(deserializer).map(Into::into)
        })]
        position: Vec3,
    },
    Tuple(
        #[serde_inline(deserialize_with = |deserializer| {
            RawVec3::deserialize(deserializer).map(Into::into)
        })]
        Vec3,
    ),
}

#[test]
fn test_deserialize() {
    let data = r#"{"position": "1 2 3"}"#;
    let data2 = "{}";
    let example: Example = serde_json::from_str(data).unwrap();
    let example_default: Example = serde_json::from_str(data2).unwrap();
    assert_eq!(example.position.0.x, 1);
    assert_eq!(example.position.0.y, 2);
    assert_eq!(example.position.0.z, 3);
    assert_eq!(example_default.position.0.x, 0);
    assert_eq!(example_default.position.0.y, 0);
    assert_eq!(example_default.position.0.z, 0);
}

#[test]
fn test_deserialize_tuple_struct_unnamed_field() {
    let data = r#"{ "inner": "1 2 3" }"#;
    let value: TupleExampleWrapper = serde_json::from_str(data).unwrap();
    assert_eq!(value.inner.0.0.x, 1);
    assert_eq!(value.inner.0.0.y, 2);
    assert_eq!(value.inner.0.0.z, 3);
}

#[test]
fn test_deserialize_enum_fields() {
    let named: ExampleEnum = serde_json::from_str(r#"{"Named":{"position":"7 8 9"}}"#).unwrap();
    let tuple: ExampleEnum = serde_json::from_str(r#"{"Tuple":"4 5 6"}"#).unwrap();

    match named {
        ExampleEnum::Named { position } => {
            assert_eq!(position.0.x, 7);
            assert_eq!(position.0.y, 8);
            assert_eq!(position.0.z, 9);
        }
        _ => panic!("expected named variant"),
    }

    match tuple {
        ExampleEnum::Tuple(position) => {
            assert_eq!(position.0.x, 4);
            assert_eq!(position.0.y, 5);
            assert_eq!(position.0.z, 6);
        }
        _ => panic!("expected tuple variant"),
    }
}
