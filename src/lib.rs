#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
//!Conveniently serialize and deserialize rust types into / from their serde name.
//!
//!## Usage
//!
//!```rust
//!use serde::{Serialize, Deserialize};
//!use serde_typename::{to_str, from_str};
//!
//!/// Enums
//!
//!#[derive(Debug, PartialEq, Serialize, Deserialize)]
//!enum Enum {
//!    UnitVariant,
//!    #[serde(rename = "RENAMED")]
//!    Renamed,
//!    HoldsData(u8),
//!    HoldsDataAsTuple(u8, u8),
//!    HoldsDataAsStruct { field: u8 }
//!}
//!
//!// Safe to serialize, held data gets discarded
//!assert_eq!(to_str(&Enum::UnitVariant).unwrap(), "UnitVariant");
//!assert_eq!(to_str(&Enum::Renamed).unwrap(), "RENAMED");
//!assert_eq!(to_str(&Enum::HoldsData(0)).unwrap(), "HoldsData");
//!assert_eq!(to_str(&Enum::HoldsDataAsTuple(0, 0)).unwrap(), "HoldsDataAsTuple");
//!assert_eq!(to_str(&Enum::HoldsDataAsStruct { field: 0 }).unwrap(), "HoldsDataAsStruct");
//!
//!// Safe to deserialize since no data is held
//!assert_eq!(from_str::<Enum>("UnitVariant").unwrap(), Enum::UnitVariant);
//!assert_eq!(from_str::<Enum>("RENAMED").unwrap(), Enum::Renamed);
//!// Cant be deserialized since their data was lost during serialization
//!assert!(from_str::<Enum>("HoldsData").is_err());
//!assert!(from_str::<Enum>("HoldsDataAsTuple").is_err());
//!assert!(from_str::<Enum>("HoldsDataAsStruct").is_err());
//!
//!/// Structs
//!
//!#[derive(Debug, PartialEq, Serialize, Deserialize)]
//!struct UnitStruct;
//!assert_eq!(to_str(&UnitStruct).unwrap(), "UnitStruct");
//!assert_eq!(from_str::<UnitStruct>("UnitStruct").unwrap(), UnitStruct);
//!
//!#[derive(Debug, PartialEq, Serialize, Deserialize)]
//!struct TupleStruct(u64, u64);
//!assert_eq!(to_str(&TupleStruct(0, 0)).unwrap(), "TupleStruct");
//!assert!(from_str::<TupleStruct>("TupleStruct").is_err());
//!
//!#[derive(Debug, PartialEq, Serialize, Deserialize)]
//!struct Struct {
//!    field: u8
//!}
//!assert_eq!(to_str(&Struct { field: 123 }).unwrap(), "Struct");
//!assert!(from_str::<Struct>("Struct").is_err());
//!```

mod de;
mod error;
mod ser;

use serde::Deserialize;
use serde::Serialize;

pub use de::Deserializer;
pub(crate) use error::ErrorCode;
pub use error::{Error, Result};
pub use ser::Serializer;

/// Convert enums and structs into its variant name
///
/// Keep in mind that all data held by the value is
/// discarded and only the type name is serialized
///
/// ```rust
///use serde::{Serialize, Deserialize};
///use serde_typename::{to_str, from_str};
///
///#[derive(Debug, PartialEq, Serialize, Deserialize)]
///enum Enum {
///    NoData,
///    WithData(u8),
///}
///assert_eq!(to_str(&Enum::NoData).unwrap(), "NoData");
///assert_eq!(to_str(&Enum::WithData(1)).unwrap(), "WithData");
///
///#[derive(Debug, PartialEq, Serialize, Deserialize)]
///struct Unit;
///assert_eq!(to_str(&Unit).unwrap(), "Unit");
///
///#[derive(Debug, PartialEq, Serialize, Deserialize)]
///struct Tuple(u64, u64);
///assert_eq!(to_str(&Tuple(1, 2)).unwrap(), "Tuple");
///
///#[derive(Debug, PartialEq, Serialize, Deserialize)]
///struct Struct {
///    field: u8
///}
///assert_eq!(to_str(&Struct { field: 1 }).unwrap(), "Struct");
/// ```
pub fn to_str<T>(value: &T) -> Result<&'static str>
where
    T: Serialize,
{
    let mut serializer = ser::Serializer {};
    value.serialize(&mut serializer)
}

/// Convert a variant name back into an enum or struct if possible
///
/// Keep in mind that all target variants or structs which
/// hold data (newtype, tuple, field variants / structs)
/// can't be deserialized since data was discarded during
/// serialization
///
/// ```rust
///use serde::{Serialize, Deserialize};
///use serde_typename::{to_str, from_str};
///
///#[derive(Debug, PartialEq, Serialize, Deserialize)]
///enum Enum {
///    NoData,
///    WithData(u8),
///}
///assert_eq!(from_str::<Enum>("NoData").unwrap(), Enum::NoData);
///assert!(from_str::<Enum>("WithData").is_err());
///
///#[derive(Debug, PartialEq, Serialize, Deserialize)]
///struct Unit;
///assert_eq!(from_str::<Unit>("Unit").unwrap(), Unit);
///
///#[derive(Debug, PartialEq, Serialize, Deserialize)]
///struct Tuple(u64, u64);
///assert!(from_str::<Tuple>("Tuple").is_err());
///
///#[derive(Debug, PartialEq, Serialize, Deserialize)]
///struct Struct {
///    field: u8
///}
///assert!(from_str::<Struct>("Struct").is_err());
/// ```
pub fn from_str<'a, D>(value: &'a str) -> Result<D>
where
    D: Deserialize<'a>,
{
    let mut deserializer = de::Deserializer::new(value);
    let variant = D::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(variant)
    } else {
        Err(Error::deserialization(error::ErrorCode::TrailingCharacters))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod ser {
        use super::*;

        mod enums {
            use super::*;

            #[test]
            fn unit_variants() {
                #[derive(Serialize)]
                enum Foo {
                    Var1,
                    #[serde(rename = "VAR2")]
                    Var2,
                }

                assert_eq!(to_str(&Foo::Var1).unwrap(), "Var1");
                assert_eq!(to_str(&Foo::Var2).unwrap(), "VAR2");
            }

            #[test]
            fn newtype_variants() {
                #[derive(Serialize)]
                enum Foo {
                    Var1(()),
                    #[serde(rename = "VAR2")]
                    Var2(u32),
                }

                assert_eq!(to_str(&Foo::Var1(())).unwrap(), "Var1");
                assert_eq!(to_str(&Foo::Var2(42)).unwrap(), "VAR2");
            }

            #[test]
            fn tuple_variants() {
                #[derive(Serialize)]
                enum Foo {
                    BAz((), u64),
                    #[serde(rename = "VAR")]
                    Var((), (), ()),
                }

                assert_eq!(to_str(&Foo::BAz((), 1337)).unwrap(), "BAz");
                assert_eq!(to_str(&Foo::Var((), (), ())).unwrap(), "VAR");
            }

            #[test]
            fn struct_variants() {
                #[derive(Serialize)]
                enum Foo {
                    Var1 {
                        field: u8,
                    },
                    #[serde(rename = "Renamed")]
                    Var2 {
                        foo: &'static str,
                    },
                }

                assert_eq!(to_str(&Foo::Var1 { field: 0 }).unwrap(), "Var1");
                assert_eq!(to_str(&Foo::Var2 { foo: "BAR" }).unwrap(), "Renamed");
            }
        }

        mod structs {
            use super::*;

            #[test]
            fn unit_structs() {
                #[derive(Serialize)]
                struct Bar;

                assert_eq!(to_str(&Bar).unwrap(), "Bar");
            }

            #[test]
            fn newtype_structs() {
                #[derive(Serialize)]
                struct Bar(u64);

                assert_eq!(to_str(&Bar(42)).unwrap(), "Bar");
            }

            #[test]
            fn tuple_structs() {
                #[derive(Serialize)]
                struct Bar(u64, u64);

                assert_eq!(to_str(&Bar(1, 42)).unwrap(), "Bar");
            }

            #[test]
            fn field_structs() {
                #[derive(Serialize)]
                struct Bar {
                    field: u8,
                }

                assert_eq!(to_str(&Bar { field: 0 }).unwrap(), "Bar");
            }
        }
    }

    mod de {
        use super::*;

        mod enums {
            use super::*;

            #[test]
            fn case_sensisitive() {
                #[derive(Debug, PartialEq, Deserialize)]
                #[allow(non_camel_case_types)]
                enum Foo {
                    foo,
                    FoO,
                    FOO,
                    fOO,
                }

                assert_eq!(from_str::<Foo>("foo").unwrap(), Foo::foo);
                assert_eq!(from_str::<Foo>("FoO").unwrap(), Foo::FoO);
                assert_eq!(from_str::<Foo>("FOO").unwrap(), Foo::FOO);
                assert_eq!(from_str::<Foo>("fOO").unwrap(), Foo::fOO);
                assert!(from_str::<Foo>("Foo").is_err());
                assert!(from_str::<Foo>("fOo").is_err());
                assert!(from_str::<Foo>("foO").is_err());
                assert!(from_str::<Foo>("FOo").is_err());
            }

            #[test]
            fn space_sensisitive() {
                #[derive(Debug, PartialEq, Deserialize)]
                #[allow(non_camel_case_types)]
                enum Foo {
                    Foo,
                }

                assert_eq!(from_str::<Foo>("Foo").unwrap(), Foo::Foo);
                assert!(from_str::<Foo>("Foo ").is_err());
                assert!(from_str::<Foo>(" Foo").is_err());
                assert!(from_str::<Foo>("F oo").is_err());
                assert!(from_str::<Foo>("F o o").is_err());
            }

            #[test]
            fn unit_variants() {
                #[derive(Debug, PartialEq, Deserialize)]
                enum Foo {
                    Var1,
                    #[serde(rename = "VAR2")]
                    Var2,
                }

                assert_eq!(from_str::<Foo>("Var1").unwrap(), Foo::Var1);
                assert_eq!(from_str::<Foo>("VAR2").unwrap(), Foo::Var2);
            }

            mod impossible {
                use super::*;

                #[test]
                fn newtype_variants() {
                    #[derive(Debug, PartialEq, Deserialize)]
                    enum Foo {
                        Foo(u8),
                    }

                    assert!(from_str::<Foo>("Foo").is_err());
                }

                #[test]
                fn tuple_variants() {
                    #[derive(Debug, PartialEq, Deserialize)]
                    enum Foo {
                        BAz(u8),
                        #[serde(rename = "VAR")]
                        Var((), (), u8),
                    }

                    assert!(from_str::<Foo>("BAz").is_err());
                    assert!(from_str::<Foo>("VAR").is_err());
                }

                #[test]
                fn struct_variants() {
                    #[derive(Debug, PartialEq, Deserialize)]
                    enum Foo {
                        BAz(u8),
                        #[serde(rename = "VAR")]
                        Var((), (), u8),
                    }

                    assert!(from_str::<Foo>("BAz").is_err());
                    assert!(from_str::<Foo>("VAR").is_err());
                }
            }
        }

        mod structs {
            use super::*;

            #[test]
            #[allow(non_camel_case_types)]
            fn case_sensisitive() {
                #[derive(Debug, Deserialize, PartialEq)]
                struct foo;
                #[derive(Debug, Deserialize, PartialEq)]
                struct FoO;
                #[derive(Debug, Deserialize, PartialEq)]
                struct FOO;
                #[derive(Debug, Deserialize, PartialEq)]
                struct fOO;

                assert_eq!(from_str::<foo>("foo").unwrap(), foo);
                assert_eq!(from_str::<FoO>("FoO").unwrap(), FoO);
                assert_eq!(from_str::<FOO>("FOO").unwrap(), FOO);
                assert_eq!(from_str::<fOO>("fOO").unwrap(), fOO);

                assert!(from_str::<foo>("Foo").is_err());
                assert!(from_str::<foo>("FoO").is_err());
                assert!(from_str::<foo>("FoO").is_err());
                assert!(from_str::<foo>("FOo").is_err());

                assert!(from_str::<FoO>("Foo").is_err());
                assert!(from_str::<FoO>("foO").is_err());
                assert!(from_str::<FoO>("foo").is_err());
                assert!(from_str::<FoO>("FOo").is_err());

                assert!(from_str::<FOO>("Foo").is_err());
                assert!(from_str::<FOO>("foO").is_err());
                assert!(from_str::<FOO>("FoO").is_err());
                assert!(from_str::<FOO>("FOo").is_err());

                assert!(from_str::<fOO>("Foo").is_err());
                assert!(from_str::<fOO>("foO").is_err());
                assert!(from_str::<fOO>("foO").is_err());
                assert!(from_str::<fOO>("FOo").is_err());
            }

            #[test]
            fn space_sensisitive() {
                #[derive(Debug, Deserialize, PartialEq)]
                struct Foo;

                assert_eq!(from_str::<Foo>("Foo").unwrap(), Foo);
                assert!(from_str::<Foo>("Foo ").is_err());
                assert!(from_str::<Foo>(" Foo").is_err());
                assert!(from_str::<Foo>("F oo").is_err());
                assert!(from_str::<Foo>("F o o").is_err());
            }

            #[test]
            fn unit_struct() {
                #[derive(Debug, Deserialize, PartialEq)]
                struct Foo;

                #[derive(Debug, Deserialize, PartialEq)]
                #[serde(rename = "BAR")]
                struct Bar;

                assert_eq!(from_str::<Foo>("Foo").unwrap(), Foo);
                assert_eq!(from_str::<Bar>("BAR").unwrap(), Bar);

                assert!(from_str::<Bar>("bAR").is_err());
                assert!(from_str::<Bar>("bar").is_err());
            }

            mod impossible {
                use super::*;

                #[test]
                fn newtype_struct() {
                    #[derive(Debug, Deserialize, PartialEq)]
                    struct Foo(u8);

                    assert!(from_str::<Foo>("Foo").is_err());
                }

                #[test]
                fn field_struct() {
                    #[derive(Debug, Deserialize, PartialEq)]
                    struct Foo {
                        field: u8,
                    }

                    assert!(from_str::<Foo>("Foo").is_err());
                }
            }
        }
    }
}
