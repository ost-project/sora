use memchr::Memchr;

#[derive(Debug)]
pub(crate) struct Splitter<'a> {
    string: &'a str,
    last_end: usize,
    memchr: Memchr<'a>,
}

impl<'a> Splitter<'a> {
    pub fn new(string: &'a str, splitter: u8) -> Self {
        Self {
            string,
            memchr: memchr::memchr_iter(splitter, string.as_bytes()),
            last_end: 0,
        }
    }
}

impl<'a> Iterator for Splitter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let next_end = match self.memchr.next() {
            None => {
                if self.last_end > self.string.len() {
                    return None;
                }
                self.string.len()
            }
            Some(end) => end,
        };
        // SAFETY: next_end never > self.string.len()
        let s = unsafe { self.string.get_unchecked(self.last_end..next_end) };
        self.last_end = next_end + 1;
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::Splitter;

    #[test]
    fn test_splitter() {
        let text =
      ";;yZCTnK,IAAO5F,gBAAkB,YACzB,IAAOC,YAAcC,UACrB;IAAOC,oBAAsB,YAE7B,EAAQ,QAER,EAAQ;;cAAe";

        assert_eq!(
            Splitter::new(text, b';').collect::<Vec<_>>().join(";"),
            text
        );
        assert_eq!(
            Splitter::new(text, b',').collect::<Vec<_>>().join(","),
            text
        );
    }
}
