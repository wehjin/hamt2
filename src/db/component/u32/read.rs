use crate::space::Space;
use crate::trie::mem::value::MemValue;
use crate::trie::SpaceTrie;

pub struct Read<'a, T: Space> {
    hash_trie: &'a SpaceTrie<T>,
    bytes_left: usize,
    start_key: i32,
    u32_index: i32,
    current_u32: Option<(u32, usize)>,
}

impl<'a, T: Space> Read<'a, T> {
    pub fn new(hash_trie: &'a SpaceTrie<T>, bytes_max: usize, start_key: i32) -> Self {
        Self {
            hash_trie,
            bytes_left: bytes_max,
            start_key,
            u32_index: 0,
            current_u32: None,
        }
    }

    async fn next_u32(&mut self) -> u32 {
        let key = self.start_key + self.u32_index;
        let Ok(Some(MemValue::U32(u32))) = self.hash_trie.query_value(key).await else {
            panic!("Unexpected MemValue variant")
        };
        self.u32_index += 1;
        u32
    }

    pub async fn into_bytes(mut self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(self.bytes_left);
        while let Some(byte) = self.next().await {
            vec.push(byte);
        }
        vec
    }

    async fn next(&mut self) -> Option<u8> {
        if self.bytes_left == 0 {
            None
        } else {
            let (u32, bits_available) = match self.current_u32 {
                Some(current) => current,
                None => {
                    let u32 = self.next_u32().await;
                    (u32, 32)
                }
            };
            let take = if bits_available >= 8 {
                let take = (u32 >> 24) as u8;
                let new_bits_available = bits_available - 8;
                if new_bits_available == 0 {
                    self.current_u32 = None;
                } else {
                    self.current_u32 = Some((u32 << 8, new_bits_available));
                }
                take
            } else {
                let bits_needed = 8 - bits_available;
                let take_available = (u32 >> (24 + bits_needed)) as u8;
                let new_u32 = self.next_u32().await;
                let take_needed = (new_u32 >> (24 + bits_available)) as u8;
                let take = (take_available << bits_needed) | take_needed;
                self.current_u32 = Some((new_u32 << bits_needed, 32 - bits_needed));
                take
            };
            self.bytes_left -= 1;
            Some(take)
        }
    }
}