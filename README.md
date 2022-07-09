[![crates.io](https://img.shields.io/crates/v/serde_typename.svg)](https://crates.io/crates/serde_typename)
[![docs](https://docs.rs/serde_typename/badge.svg)](https://docs.rs/serde_typename)

# serde_typename

Conveniently serialize and deserialize rust types into / from their serde name.

## Usage

```rust
use serde::{Serialize, Deserialize};
use serde_typename::{to_str, from_str};

/// Enums

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Enum {
    UnitVariant,
    #[serde(rename = "RENAMED")]
    Renamed,
    HoldsData(u8),
    HoldsDataAsTuple(u8, u8),
    HoldsDataAsStruct { field: u8 }
}

// Safe to serialize, held data gets discarded
assert_eq!(to_str(&Enum::UnitVariant).unwrap(), "UnitVariant");
assert_eq!(to_str(&Enum::Renamed).unwrap(), "RENAMED");
assert_eq!(to_str(&Enum::HoldsData(0)).unwrap(), "HoldsData");
assert_eq!(to_str(&Enum::HoldsDataAsTuple(0, 0)).unwrap(), "HoldsDataAsTuple");
assert_eq!(to_str(&Enum::HoldsDataAsStruct { field: 0 }).unwrap(), "HoldsDataAsStruct");

// Safe to deserialize since no data is held
assert_eq!(from_str::<Enum>("UnitVariant").unwrap(), Enum::UnitVariant);
assert_eq!(from_str::<Enum>("RENAMED").unwrap(), Enum::Renamed);
// Cant be deserialized since their data was lost during serialization
assert!(from_str::<Enum>("HoldsData").is_err());
assert!(from_str::<Enum>("HoldsDataAsTuple").is_err());
assert!(from_str::<Enum>("HoldsDataAsStruct").is_err());

/// Structs

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct UnitStruct;
assert_eq!(to_str(&UnitStruct).unwrap(), "UnitStruct");
assert_eq!(from_str::<UnitStruct>("UnitStruct").unwrap(), UnitStruct);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TupleStruct(u64, u64);
assert_eq!(to_str(&TupleStruct(0, 0)).unwrap(), "TupleStruct");
assert!(from_str::<TupleStruct>("TupleStruct").is_err());

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Struct {
    field: u8
}
assert_eq!(to_str(&Struct { field: 123 }).unwrap(), "Struct");
assert!(from_str::<Struct>("Struct").is_err());
```

## Acknowledgement

This crate originated as a fork of `serde_variant` which is maintained by
Daniel Mueller `deso@posteo.net`
