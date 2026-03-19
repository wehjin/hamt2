use crate::core::{Attr, Ent, Tx, Val};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("InvalidEAVTKey: {0}")]
    InvalidEAVTKey(String),

    #[error("InvalidVKey: {0}")]
    InvalidVKey(String),

    #[error("InvalidVKeyType: {0}")]
    InvalidValType(String),

    #[error("ParseInt: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
}

pub fn eavt_full_key(e: &Ent, a: &Attr, v: &Val, t: &Tx) -> String {
    format!("eavt:{}{}{}{}", e_key(e), a_key(a), v_key(v), t_key(t))
}

pub fn val_from_eavt_full_key(eavt: &str) -> Result<Val, Error> {
    if eavt.starts_with("eavt:") {
        let eavt = &eavt[5..];
        let parts: Vec<&str> = eavt.split(':').collect();
        let v_key = *parts
            .get(2)
            .ok_or(Error::InvalidEAVTKey(eavt.to_string()))?;
        val_from_v_key(v_key)
    } else {
        Err(Error::InvalidEAVTKey(eavt.to_string()))
    }
}

pub fn eavt_ea_key(e: &Ent, a: &Attr) -> String {
    format!("eavt:{}{}", e_key(e), a_key(a))
}

pub fn aevt_key(a: &Attr, e: &Ent, v: &Val, t: &Tx) -> String {
    format!("aevt:{}{}{}{}", a_key(a), e_key(e), v_key(v), t_key(t))
}

pub fn avet_key(a: &Attr, v: &Val, e: &Ent, t: &Tx) -> String {
    format!("avet:{}{}{}{}", a_key(a), v_key(v), e_key(e), t_key(t))
}

pub fn t_key(t: &Tx) -> String {
    format!("tx{}:", t.0)
}

pub fn v_key(v: &Val) -> String {
    match v {
        Val::Uint(v) => format!("val/ui{}:", *v),
    }
}

pub fn val_from_v_key(v_key: &str) -> Result<Val, Error> {
    if v_key.starts_with("val/") {
        let typed_str = &v_key[4..];
        if typed_str.starts_with("ui") {
            let uint_str = &typed_str[2..];
            let uint = uint_str.parse::<u64>()?;
            Ok(Val::Uint(uint))
        } else {
            Err(Error::InvalidValType(typed_str.to_string()))
        }
    } else {
        Err(Error::InvalidVKey(v_key.to_string()))
    }
}

pub fn a_key(a: &Attr) -> String {
    format!("attr{}:", a.0)
}

pub fn e_key(e: &Ent) -> String {
    format!("ent{}:", e.0)
}
