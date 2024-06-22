use crate::{ParseError, ParseResult};
use std::io;
use std::io::Write;

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const BASE64_VALUES: [i8; 256] = get_base64_map();

const fn get_base64_map() -> [i8; 256] {
    let mut res = [-1i8; 256];
    // `for in` is not allowed in const fn
    let mut idx = 0;
    while idx < 64 {
        res[BASE64_CHARS[idx] as usize] = idx as i8;
        idx += 1;
    }
    res
}

#[derive(Debug)]
pub(crate) struct VlqDecoder {
    buf: [i64; 5],
}

impl VlqDecoder {
    pub fn new() -> Self {
        Self { buf: [0; 5] }
    }

    pub fn decode(&mut self, segment: &str) -> ParseResult<&[i64]> {
        let mut len = 0;

        let mut cur_value = 0;
        let mut shift = 0;

        for byte in segment.bytes() {
            let value = BASE64_VALUES[byte as usize] as i64;
            let val = value & 0b11111;
            cur_value += val
                .checked_shl(shift)
                .ok_or_else(|| ParseError::MappingMalformed(segment.to_owned()))?;
            shift += 5;

            if value & 0b100000 == 0 {
                if len > 4 {
                    return Err(ParseError::MappingMalformed(segment.to_owned()));
                }

                let is_negative = (cur_value & 1) == 1;
                cur_value >>= 1;
                if is_negative {
                    cur_value = -cur_value;
                }
                self.buf[len] = cur_value;
                len += 1;
                cur_value = 0;
                shift = 0;
            }
        }

        if shift != 0 || !matches!(len, 1 | 4 | 5) {
            Err(ParseError::MappingMalformed(segment.to_owned()))
        } else {
            // SAFETY: self.len is guaranteed to be <= 5 in the above code
            Ok(unsafe { self.buf.get_unchecked(..len) })
        }
    }
}

#[derive(Debug)]
pub(crate) struct VlqEncoder<'a, W>
where
    W: Write,
{
    writer: &'a mut W,
}

impl<'a, W> VlqEncoder<'a, W>
where
    W: Write,
{
    pub fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }

    pub fn encode(&mut self, prev: u32, cur: u32) -> io::Result<()> {
        let delta = cur as i64 - prev as i64;

        let mut num = if delta < 0 {
            ((-delta) << 1) + 1
        } else {
            delta << 1
        } as usize;

        loop {
            let mut digit = num & 0b11111;
            num >>= 5;
            if num != 0 {
                digit |= 1 << 5;
            }
            self.writer.write_all(&[BASE64_CHARS[digit]])?;
            if num == 0 {
                break;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{VlqDecoder, VlqEncoder};
    use crate::ParseError;

    fn encode_helper(vlq: &[i64]) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut encoder = VlqEncoder::new(&mut buf);
        for &num in vlq {
            encoder.encode(0, num as u32).unwrap();
        }
        buf
    }

    #[test]
    fn test_vlq_decode_encode_normal() {
        let mut decoder = VlqDecoder::new();
        assert_eq!(&encode_helper(decoder.decode("AAAA").unwrap()), b"AAAA");
        assert_eq!(&encode_helper(decoder.decode("Q").unwrap()), b"Q");
    }

    #[test]
    fn test_vlq_decode_malformed() {
        let mut decoder = VlqDecoder::new();
        assert!(matches!(
            decoder.decode("aAC5B9iiC/"),
            Err(ParseError::MappingMalformed(..))
        ));
        assert!(matches!(
            decoder.decode("你好"),
            Err(ParseError::MappingMalformed(..))
        ));
        assert!(matches!(
            decoder.decode(""),
            Err(ParseError::MappingMalformed(..))
        ));
        // overflow
        assert!(matches!(
            decoder.decode("AAAAAAAAAAA"),
            Err(ParseError::MappingMalformed(..))
        ));
    }
}
