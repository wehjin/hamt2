use crate::{Base, Buffer, BufferIndex, BufferMut, ByteData, MapBase, Slot};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct VecBuffer {
    past: Option<Arc<VecBuffer>>,
    slots: Vec<Slot>,
}

impl VecBuffer {
    pub fn new(seed: Slot) -> Self {
        Self {
            past: None,
            slots: vec![seed],
        }
    }
    pub fn extend(past: Arc<Self>) -> Self {
        Self {
            past: Some(past),
            slots: vec![],
        }
    }
    pub fn retract(self) -> Self {
        if let Some(past) = self.past {
            let mut slots = past.deref().slots.clone();
            slots.extend(self.slots);
            debug_assert!(matches!(dbg!(slots.last()), Some(Slot::MapBase(_))));
            Self { past: None, slots }
        } else {
            self
        }
    }
    pub fn len(&self) -> usize {
        self.slots.len() + self.past_len()
    }
    fn past_len(&self) -> usize {
        if let Some(past) = &self.past {
            past.slots.len()
        } else {
            0usize
        }
    }
}

impl Buffer for VecBuffer {
    fn get_root(&self) -> MapBase {
        self.get_subtrie(BufferIndex(self.len() as i32 - 1))
    }

    fn get_base(&self, id: BufferIndex, size: usize) -> &[Slot] {
        if id.0 < 0 {
            return &[];
        }
        let mut start = id.0 as usize;
        if let Some(past) = &self.past {
            let past_len = past.slots.len();
            if start < past_len {
                return past.get_base(id, size);
            }
            start -= past_len;
        }
        if start >= self.slots.len() {
            return &[];
        }
        let end = start + size;
        debug_assert!(end <= self.slots.len(), "too few slots in the buffer");
        &self.slots[start..end]
    }

    fn get_bytes(&self, index: BufferIndex) -> Vec<u8> {
        let mut cursor = ByteCursor::new(index, self);
        let len = {
            let mut len: usize = 0;
            let mut shift: usize = 0;
            loop {
                let byte = cursor.next_byte(self);
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
            let byte = cursor.next_byte(self);
            vec.push(byte);
        }
        vec
    }
}

impl BufferMut for VecBuffer {
    async fn push_root(&mut self, root: MapBase) {
        self.push_subtrie(root).await;
    }

async fn push_base(&mut self, base: Base) -> BufferIndex {
        let index = BufferIndex(self.len() as i32);
        self.slots.extend(base.slots);
        index
    }

    async fn push_bytes(&mut self, bytes: impl AsRef<[u8]>) -> BufferIndex {
        let base = build_base_with_bytes(bytes);
        self.push_base(base).await
    }
}

fn build_base_with_bytes(bytes: impl AsRef<[u8]>) -> Base {
    let bytes = bytes.as_ref();
    let mut data = Vec::new();
    let mut len = bytes.len();
    while len >= 0x80 {
        data.push((len & 0x7F | 0x80) as u8);
        len >>= 7;
    }
    data.push(len as u8);
    data.extend_from_slice(bytes);
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

struct ByteCursor {
    start_index: BufferIndex,
    slot_offset: usize,
    byte_offset: usize,
    byte_data: ByteData,
}

impl ByteCursor {
    fn new(start_index: BufferIndex, buffer: &VecBuffer) -> Self {
        Self {
            start_index,
            slot_offset: 0,
            byte_offset: 0,
            byte_data: get_byte_data_from_buffer(start_index, buffer),
        }
    }
    fn next_byte(&mut self, buffer: &VecBuffer) -> u8 {
        if self.byte_offset >= 8 {
            self.slot_offset += 1;
            self.byte_offset = 0;
            self.byte_data = get_byte_data_from_buffer(self.start_index + self.slot_offset, buffer);
        }
        let byte = self.byte_data[self.byte_offset];
        self.byte_offset += 1;
        byte
    }
}

fn get_byte_data_from_buffer(index: BufferIndex, buffer: &VecBuffer) -> ByteData {
    let base = buffer.get_base(index, 1);
    let slot = base.first().expect("slot not found");
    let Slot::ByteData(byte_data) = slot else {
        unreachable!("slot should contain byte-data");
    };
    byte_data.clone()
}

#[cfg(test)]
mod tests {
    use crate::{Buffer, BufferMut, SkyTrieMut};

    #[tokio::test]
    async fn buffer_pushes_and_gets_bytes() {
        let mut trie = SkyTrieMut::new();
        let mut bytes = b"hello"[..].to_vec();
        for _ in 0..7 {
            let index = trie.push_bytes(&bytes).await;
            let get_bytes = trie.get_bytes(index);
            assert_eq!(get_bytes, bytes);
            bytes.extend(bytes.clone());
        }
    }
}
