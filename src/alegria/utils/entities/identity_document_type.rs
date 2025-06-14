use std::fmt::Display;

use iced::widget::text::IntoFragment;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, Postgres, Type, postgres::PgTypeInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IdentityDocumentType {
    #[default]
    Dni,
    Nie,
    Nif,
    Pasaporte,
    CarnetConducir,
}

impl Display for IdentityDocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            IdentityDocumentType::Dni => write!(f, "DNI"),
            IdentityDocumentType::Nie => write!(f, "NIE"),
            IdentityDocumentType::Nif => write!(f, "NIF"),
            IdentityDocumentType::Pasaporte => write!(f, "Pasaporte"),
            IdentityDocumentType::CarnetConducir => write!(f, "Carnet de Conducir"),
        }
    }
}

impl<'a> IntoFragment<'a> for IdentityDocumentType {
    fn into_fragment(self) -> iced::widget::text::Fragment<'a> {
        iced::widget::text::Fragment::Owned(self.to_string())
    }
}

impl IdentityDocumentType {
    pub const ALL: &'static [Self] = &[
        Self::Dni,
        Self::Nie,
        Self::Nif,
        Self::Pasaporte,
        Self::CarnetConducir,
    ];

    pub fn to_id(self) -> i32 {
        match self {
            IdentityDocumentType::Dni => 1,
            IdentityDocumentType::Nie => 2,
            IdentityDocumentType::Nif => 3,
            IdentityDocumentType::Pasaporte => 4,
            IdentityDocumentType::CarnetConducir => 5,
        }
    }

    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(IdentityDocumentType::Dni),
            2 => Some(IdentityDocumentType::Nie),
            3 => Some(IdentityDocumentType::Nif),
            4 => Some(IdentityDocumentType::Pasaporte),
            5 => Some(IdentityDocumentType::CarnetConducir),
            _ => None,
        }
    }
}

// Implement Type trait to tell SQLx how to handle this type
impl Type<Postgres> for IdentityDocumentType {
    fn type_info() -> PgTypeInfo {
        <i32 as Type<Postgres>>::type_info()
    }
}

// Implement Encode to convert enum to database value
impl<'q> Encode<'q, Postgres> for IdentityDocumentType {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i32 as Encode<Postgres>>::encode_by_ref(&self.to_id(), buf)
    }
}

// Implement Decode to convert database value to enum
impl<'r> Decode<'r, Postgres> for IdentityDocumentType {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let id = <i32 as Decode<Postgres>>::decode(value)?;
        Self::from_id(id).ok_or_else(|| format!("Invalid identity document type id: {id}").into())
    }
}
