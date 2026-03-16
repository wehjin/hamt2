use crate::hamt::segment::{Segment, SegmentIndex};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Value {
    Uint(u32),
}

impl Value {
    const UINT: u8 = 1;
    pub fn to_be_bytes(&self) -> Vec<u8> {
        match self {
            Value::Uint(value) => {
                let mut bytes = vec![Self::UINT];
                bytes.extend_from_slice(&value.to_be_bytes());
                bytes
            }
        }
    }
    pub fn from_be_bytes(start: SegmentIndex, segment: &Segment) -> (Self, SegmentIndex) {
        let (value_type, start) = segment.read_u8(start);
        match value_type {
            Self::UINT => {
                let (value, start) = segment.read_u32(start);
                (Value::Uint(value), start)
            }
            _ => panic!("Invalid value type"),
        }
    }
}