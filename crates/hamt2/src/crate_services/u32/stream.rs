pub struct Stream<'a> {
    bytes: &'a [u8],
    start_key: i32,
    used_bytes: usize,
    used_bits: usize,
    max_bit_index: usize,
    next_key: i32,
}

impl<'a> Stream<'a> {
    pub fn new(bytes: &'a [u8], start_key: i32) -> Self {
        Self {
            bytes,
            start_key,
            used_bytes: 0,
            used_bits: 0,
            max_bit_index: bytes.len() * 8,
            next_key: 0,
        }
    }
}

impl Iterator for Stream<'_> {
    type Item = (i32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        let total_used_bits = self.used_bytes * 8 + self.used_bits;
        if total_used_bits >= self.max_bit_index {
            None
        } else {
            let mut next = 0u32;
            let mut needed = 32;
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
            let key = self.next_key + self.start_key;
            self.next_key += 1;
            Some((key, next))
        }
    }
}
