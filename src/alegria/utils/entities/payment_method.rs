use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, Postgres, Type, postgres::PgTypeInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PaymentMethod {
    #[default]
    Efectivo,
    Tarjeta,
    Adeudo,
}

impl Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            PaymentMethod::Efectivo => write!(f, "Efectivo"),
            PaymentMethod::Tarjeta => write!(f, "Tarjeta"),
            PaymentMethod::Adeudo => write!(f, "Adeudo"),
        }
    }
}

impl PaymentMethod {
    pub fn to_id(self) -> i32 {
        match self {
            PaymentMethod::Efectivo => 1,
            PaymentMethod::Tarjeta => 2,
            PaymentMethod::Adeudo => 3,
        }
    }

    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(PaymentMethod::Efectivo),
            2 => Some(PaymentMethod::Tarjeta),
            3 => Some(PaymentMethod::Adeudo),
            _ => None,
        }
    }
}

// Implement Type trait to tell SQLx how to handle this type
impl Type<Postgres> for PaymentMethod {
    fn type_info() -> PgTypeInfo {
        <i32 as Type<Postgres>>::type_info()
    }
}

// Implement Encode to convert enum to database value
impl<'q> Encode<'q, Postgres> for PaymentMethod {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i32 as Encode<Postgres>>::encode_by_ref(&self.to_id(), buf)
    }
}

// Implement Decode to convert database value to enum
impl<'r> Decode<'r, Postgres> for PaymentMethod {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let id = <i32 as Decode<Postgres>>::decode(value)?;
        Self::from_id(id).ok_or_else(|| format!("Invalid payment_method id: {id}").into())
    }
}
