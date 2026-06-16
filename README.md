# serde_inline

`serde_inline` is a small proc-macro crate for inlining serde field behavior directly on a struct field.

It lets you attach `deserialize_with` and `default` logic inside a field-level `#[serde_inline(...)]` attribute, while the macro rewrites that into the `#[serde(...)]` attribute that serde understands.

## Why this exists

Serde usually expects helper functions or standalone `#[serde(...)]` items for custom field behavior. This crate keeps the logic close to the field itself, which can make complex model types easier to read.

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
serde_inline = "0.1"

[dependencies.serde]
version = "1"
features = ["derive"]
```

## Usage

Apply `#[serde_inline]` to the struct or enum, then use `#[serde_inline(...)]` on the fields you want to customize.

```rust
#[serde_inline]
#[derive(Deserialize)]
struct Example {
	#[serde_inline(
		deserialize_with = |deserializer| {
            RawVec3::deserialize(deserializer).map(Into::into)
        },
		default = Vec3(RawVec3 { x: 0, y: 0, z: 0 })
	)]
	position: Vec3,
}
```

## Supported field options

- `deserialize_with = <expr>`: inlined expression that receives the deserializer and returns the field value.
- `default = <expr>`: inlined expression used as the field default.

## Limitations

- Unsupported `#[serde_inline(...)]` keys will cause a macro error.
- The crate is focused on serde deserialization helpers; it does not add new serde features beyond field-level `deserialize_with` and `default`.

## Example

See [tests/inline.rs](tests/inline.rs) for a complete working example.
