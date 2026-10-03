use crate::{Base, Buffer, BufferIndex, BufferMut, Slot};
use serde::{Deserialize, Serialize};
use std::ops::Index;

#[cfg(test)]
mod tests {
    use crate::SkyTrieMut;
    use crate::{get_bytes_from_buffer, push_bytes_to_buffer};

    #[tokio::test]
    async fn buffer_pushes_and_gets_bytes() {
        let mut buffer = SkyTrieMut::new();
        let mut bytes = b"hello"[..].to_vec();
        for _ in 0..7 {
            let index = push_bytes_to_buffer(&bytes, &mut buffer).await;
            let get_bytes = get_bytes_from_buffer(index, &mut buffer).await;
            assert_eq!(get_bytes, bytes);
            bytes.extend(bytes.clone());
        }
    }
}

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

pub async fn push_bytes_to_buffer(
    bytes: impl AsRef<[u8]>,
    buffer: &mut impl BufferMut,
) -> BufferIndex {
    let base = build_base_with_bytes(bytes);
    buffer.push_base(base).await
}

fn build_base_with_bytes(bytes: impl AsRef<[u8]>) -> Base {
    let bytes = bytes.as_ref();
    let mut data = Vec::new();
    // Add bytes' length and content to data.
    let mut len = bytes.len();
    while len >= 0x80 {
        data.push((len & 0x7F | 0x80) as u8);
        len >>= 7;
    }
    data.push(len as u8);
    data.extend_from_slice(bytes);
    // Extend data to a multiple of 8;
    let remainder = data.len() % 8;
    if remainder != 0 {
        data.resize(data.len() + (8 - remainder), 0);
    }
    let mut base = Base { slots: vec![] };
    let slot_count = data.len() / 8;
    for i in 0..slot_count {
        let byte_data = ByteData::from(&data[i * 8..(i + 1) * 8]);
        base.slots.push(Slot::ByteData(byte_data));
    }
    base
}
pub async fn get_bytes_from_buffer(index: BufferIndex, buffer: &impl Buffer) -> Vec<u8> {
    let mut cursor = ByteCursor::new(index, buffer).await;
    let len = {
        let mut len: usize = 0;
        let mut shift: usize = 0;
        loop {
            let byte = cursor.next_byte(buffer).await;
            len |= ((byte & 0x7F) as usize) << shift;
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
        }
        len
    };
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        let byte = cursor.next_byte(buffer).await;
        vec.push(byte);
    }
    vec
}

struct ByteCursor {
    start_index: BufferIndex,
    slot_offset: usize,
    byte_offset: usize,
    byte_data: ByteData,
}

impl ByteCursor {
    pub async fn new(start_index: BufferIndex, buffer: &impl Buffer) -> Self {
        Self {
            start_index,
            slot_offset: 0,
            byte_offset: 0,
            byte_data: get_byte_data_from_buffer(start_index, buffer).await,
        }
    }
    pub async fn next_byte(&mut self, buffer: &impl Buffer) -> u8 {
        if self.byte_offset >= 8 {
            self.slot_offset += 1;
            self.byte_offset = 0;
            self.byte_data =
                get_byte_data_from_buffer(self.start_index + self.slot_offset, buffer).await;
        }
        let byte = self.byte_data[self.byte_offset];
        self.byte_offset += 1;
        byte
    }
}

async fn get_byte_data_from_buffer(index: BufferIndex, buffer: &impl Buffer) -> ByteData {
    let mut base = buffer.get_base(index, 1).await;
    let slot = base.slots.pop().expect("slot not found");
    let Slot::ByteData(byte_data) = slot else {
        unreachable!("slot should contain byte-data");
    };
    byte_data
}
