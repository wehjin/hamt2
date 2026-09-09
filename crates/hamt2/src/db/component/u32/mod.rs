mod read;
mod stream;

pub use read::*;
pub use stream::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_works() {
        let bytes = [0u8, 0, 0, 3];
        let streamer = Stream::new(&bytes, 0);
        let key_values = streamer.collect::<Vec<_>>();
        assert_eq!(vec![(0, 3)], key_values);
    }

    #[test]
    fn fewer_than_32_bits() {
        let bytes = [0xffu8];
        let streamer = Stream::new(&bytes, 0);
        let key_values = streamer.collect::<Vec<_>>();
        assert_eq!(vec![(0, 0xff00_0000)], key_values);
    }

    #[test]
    fn more_32_bits() {
        let bytes = [0xffu8, 0xffu8, 0xffu8, 0xffu8, 0x88u8];
        let streamer = Stream::new(&bytes, 0);
        let key_values = streamer.collect::<Vec<_>>();
        assert_eq!(vec![(0, 0xffff_ffff), (1, 0x8800_0000),], key_values);
    }

    #[test]
    fn bytes_fill_exactly_32_bits() {
        let bytes = [0xffu8; 32];
        let streamer = Stream::new(&bytes, 0);
        let key_values = streamer.collect::<Vec<_>>();
        let expected = [0xffffffff; 8]
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as i32, v))
            .collect::<Vec<_>>();
        assert_eq!(expected, key_values);
    }
}
