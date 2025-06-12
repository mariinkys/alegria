use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, Postgres, Type, postgres::PgTypeInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Other,
}

impl Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Gender::Male => write!(f, "Hombre"),
            Gender::Female => write!(f, "Mujer"),
            Gender::Other => write!(f, "Otros"),
        }
    }
}

impl Gender {
    pub const ALL: &'static [Self] = &[Self::Male, Self::Female, Self::Other];

    pub fn to_id(self) -> i32 {
        match self {
            Gender::Male => 1,
            Gender::Female => 2,
            Gender::Other => 3,
        }
    }

    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Gender::Male),
            2 => Some(Gender::Female),
            3 => Some(Gender::Other),
            _ => None,
        }
    }
}

// Implement Type trait to tell SQLx how to handle this type
impl Type<Postgres> for Gender {
    fn type_info() -> PgTypeInfo {
        <i32 as Type<Postgres>>::type_info()
    }
}

// Implement Encode to convert enum to database value
impl<'q> Encode<'q, Postgres> for Gender {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i32 as Encode<Postgres>>::encode_by_ref(&self.to_id(), buf)
    }
}

// Implement Decode to convert database value to enum
impl<'r> Decode<'r, Postgres> for Gender {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let id = <i32 as Decode<Postgres>>::decode(value)?;
        Self::from_id(id).ok_or_else(|| format!("Invalid gender id: {id}").into())
    }
}
