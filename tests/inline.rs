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
