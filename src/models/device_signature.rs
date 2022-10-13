use diesel::pg::Pg;
use diesel::serialize::ToSql;
use diesel::sql_types::Bytea;
use diesel::AsExpression;

#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
#[derive(AsExpression)]
#[diesel(sql_type = Bytea)]
pub struct DeviceSignature(SignitureBits);

#[repr(C)]
#[derive(Clone, Copy, Eq)]
union SignitureBits {
    integer: u128,
    bytes: [u8; 16]
}

impl PartialEq for SignitureBits {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            <u128 as PartialEq>::eq(&self.integer, &other.integer)
        }
    }

    fn ne(&self, other: &Self) -> bool {
        unsafe {
            <u128 as PartialEq>::ne(&self.integer, &other.integer)
        }
    }
}

impl core::fmt::Debug for DeviceSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

impl std::fmt::Display for DeviceSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::traits::ToHexadecimal;
        f.write_str(unsafe { &self.0.integer.to_hexadecimal() })
    }
}

impl<'a> ToSql<Bytea, Pg> for DeviceSignature where &'a [u8]: ToSql<Bytea, Pg> {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        <[u8] as ToSql::<Bytea, Pg>>::to_sql(unsafe { &self.0.bytes }, out)
    }
}

impl<'a> serde::Serialize for DeviceSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        use crate::traits::ToHexadecimal;
        serializer.serialize_str(unsafe { &self.0.integer.to_hexadecimal() })
    }
}

impl<'de> serde::Deserialize<'de> for DeviceSignature where String: serde::Deserialize<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        use crate::traits::FromHexadecimal;
        let hex = String::deserialize(deserializer)?;
        let integer = u128::from_hexadecimal(&hex)
            .or_else(|err| 
                Err(serde::de::Error::custom(err.to_string()))
            )?;
        
        Ok(Self(SignitureBits { integer }))
    }
}