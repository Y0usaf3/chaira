use crate::prelude::*;
use crate::{bitmask_serde, relation};
use bitmask::bitmask;
use surrealdb_types::{Error, Value};

bitmask! {
    pub mask BasePermissions: i32 where flags BasePermission {
        Admin = 1 << 0,
        View = 1 << 1,
        Edit = 1 << 2,
        Delete = 1 << 3,

        ManageTables = 1 << 4,
        ManageViews = 1 << 5,
        ManagerUserPermissions = 1 << 6,
        ManageAutomatisations = 1 << 7,
        ManageInvitations = 1 << 8,
    }
}
bitmask_serde!(BasePermissions);
relation!(CanAccessBase, BasePermissions);

bitmask! {
    pub mask TablePermissions: i32 where flags TablePermission {
        Admin = 1 << 0,
        View = 1 << 1,
        Edit = 1 << 2,
        Delete = 1 << 3,

        BulkImport = 1 << 4,
        LockFields = 1 << 5,
        Export = 1 << 6,
        Archive = 1 << 7
    }
}
bitmask_serde!(TablePermissions);
relation!(CanAccessTable, TablePermissions);

bitmask! {
    pub mask FieldPermissions: i32 where flags FieldPermission {
        Admin = 1 << 0,
        View = 1 << 1,
        Edit = 1 << 2,
        Delete = 1 << 3,

        Comment = 1 << 4,
        Lock = 1 << 5,
        // XXXXXX = 1 << 6,
        // XXXXXX = 1 << 7
    }
}
bitmask_serde!(FieldPermissions);
relation!(CanAccessField, FieldPermissions);

#[macro_export]
macro_rules! relation {
    ( $( $x:ident ,$y:ident), * ) => {
        #[derive(Deserialize, Serialize, PartialEq, Eq, SurrealValue)]
        $(pub struct $x {
            pub perm: $y,
        })*
    };
}

#[macro_export]
macro_rules! bitmask_serde {
    ($ty:ident) => {
        impl serde::Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_i32(self.mask)
            }
        }

        impl<'de> serde::Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let mask = i32::deserialize(deserializer)?;
                Ok($ty { mask })
            }
        }

        impl From<i32> for $ty {
            fn from(mask: i32) -> Self {
                $ty { mask }
            }
        }

        impl From<$ty> for i32 {
            fn from(val: $ty) -> i32 {
                val.mask
            }
        }

        impl std::fmt::Debug for $ty {
            fn fmt(
                &self,
                _: &mut std::fmt::Formatter<'_>,
            ) -> std::result::Result<(), std::fmt::Error> {
                Ok(())
            }
        }

        impl SurrealValue for $ty {
            fn kind_of() -> Kind {
                Kind::Number
            }

            fn into_value(self) -> Value {
                Value::Number(Number::Int(self.mask as i64))
            }

            fn from_value(value: Value) -> Result<Self, Error> {
                match value {
                    Value::Number(num) => match num {
                        Number::Int(i) => Ok($ty { mask: i as i32 }),
                        Number::Float(f) => Ok($ty { mask: f as i32 }),
                        _ => Err(Error::thrown(
                            "Unsupported number type for bitmask".to_string(),
                        )),
                    },
                    _ => Err(Error::thrown(
                        "Expected a numeric value for bitmask".to_string(),
                    )),
                }
            }

            fn is_value(value: &Value) -> bool {
                matches!(value, Value::Number(_))
            }
        }
    };
}
