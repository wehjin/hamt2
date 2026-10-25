use serde::{Deserialize, Serialize};
use std::ops::Index;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ByteData(pub [u8; 8]);
impl From<&[u8]> for ByteData {
    fn from(slice: &[u8]) -> ByteData {
        ByteData([
            slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
        ])
    }
}

impl Index<usize> for ByteData {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
