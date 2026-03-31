use crate::space::Space;
use crate::trie::mem::value::MemValue;
use crate::trie::space::trie::SpaceTrie;

pub struct U31Streamer<'a> {
    bytes: &'a [u8],
    used_bytes: usize,
    used_bits: usize,
    max_bit_index: usize,
    next_key: i32,
}

impl<'a> U31Streamer<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            used_bytes: 0,
            used_bits: 0,
            max_bit_index: bytes.len() * 8,
            next_key: 0,
        }
    }
}

impl Iterator for U31Streamer<'_> {
    type Item = (i32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        let total_used_bits = self.used_bytes * 8 + self.used_bits;
        if total_used_bits >= self.max_bit_index {
            None
        } else {
            let mut next = 0u32;
            let mut needed = 31;
            while needed > 0 {
                if self.used_bits > 0 {
                    let unused_bits = 8 - self.used_bits;
                    let take_bits = needed.min(unused_bits);
                    let new_used_bits = self.used_bits + take_bits;
                    let new_unused_bits = 8 - new_used_bits;
                    let byte = self.bytes[self.used_bytes];
                    let addition = (byte & (0xffu8 >> self.used_bits)) >> new_unused_bits;
                    next = (next << take_bits) | (addition as u32);
                    needed -= take_bits;
                    if new_unused_bits == 0 {
                        self.used_bytes += 1;
                        self.used_bits = 0;
                    } else {
                        self.used_bits = new_used_bits;
                    }
                } else {
                    if self.used_bytes == self.bytes.len() {
                        next = next << needed;
                        needed = 0;
                    } else {
                        if needed >= 8 {
                            let addition = self.bytes[self.used_bytes];
                            self.used_bytes += 1;
                            next = (next << 8) | (addition as u32);
                            needed -= 8;
                        } else {
                            let take_width = needed;
                            let addition = self.bytes[self.used_bytes] >> (8 - take_width);
                            next = (next << take_width) | (addition as u32);
                            needed -= take_width;
                            self.used_bits = take_width;
                        }
                    }
                }
            }
            let key = self.next_key;
            self.next_key += 1;
            Some((key, next))
        }
    }
}

pub struct U31Builder<'a, T: Space> {
    hash_trie: &'a SpaceTrie<T>,
    bytes_left: usize,
    u31_index: i32,
    current_u32: Option<(u32, usize)>,
}

impl<'a, T: Space> U31Builder<'a, T> {
    pub fn new(hash_trie: &'a SpaceTrie<T>, bytes_max: usize) -> Self {
        Self {
            hash_trie,
            bytes_left: bytes_max,
            u31_index: 0,
            current_u32: None,
        }
    }

    async fn next_u31(&mut self) -> u32 {
        let Ok(Some(MemValue::U32(u31))) = self.hash_trie.query_value(self.u31_index).await else {
            panic!("Unexpected MemValue variant")
        };
        self.u31_index += 1;
        u31
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
                    let u31 = self.next_u31().await;
                    (u31 << 1, 31)
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
                let u31 = self.next_u31().await;
                let new_u32 = u31 << 1;
                let take_needed = (new_u32 >> (24 + bits_available)) as u8;
                let take = (take_available << bits_needed) | take_needed;
                self.current_u32 = Some((new_u32 << bits_needed, 31 - bits_needed));
                take
            };
            self.bytes_left -= 1;
            Some(take)
        }
    }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
    fn boundary_works() {
        let bytes = [0u8, 0, 0, 2];
        let streamer = U31Streamer::new(&bytes);
        let key_values = streamer.collect::<Vec<_>>();
        assert_eq!(vec![(0, 1), (1, 0)], key_values);
    }

    #[test]
    fn fewer_than_31_bits() {
        let bytes = [0xffu8];
        let streamer = U31Streamer::new(&bytes);
        let key_values = streamer.collect::<Vec<_>>();
        assert_eq!(vec![(0, 0x7f800000)], key_values);
    }

    #[test]
    fn more_31_bits() {
        let bytes = [0xffu8, 0xffu8, 0xffu8, 0xffu8];
        let streamer = U31Streamer::new(&bytes);
        let key_values = streamer.collect::<Vec<_>>();
        assert_eq!(vec![(0, 0x7fffffff), (1, 0x40000000),], key_values);
    }

    #[test]
    fn bytes_fill_exactly_31_bits() {
        let bytes = [0xffu8; 31];
        let streamer = U31Streamer::new(&bytes);
        let key_values = streamer.collect::<Vec<_>>();
        let expected = [0x7fffffff; 8]
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as i32, v))
            .collect::<Vec<_>>();
        assert_eq!(expected, key_values);
    }
}
