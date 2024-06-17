use memchr::Memchr2;

#[derive(Debug)]
pub(crate) struct MappingSplitter<'a> {
    string: &'a str,
    cur_start: usize,
    memchr: Memchr2<'a>,
}

impl<'a> MappingSplitter<'a> {
    pub fn new(string: &'a str) -> Self {
        Self {
            string,
            memchr: memchr::memchr2_iter(b';', b',', string.as_bytes()),
            cur_start: 0,
        }
    }
}

impl<'a> Iterator for MappingSplitter<'a> {
    // segment, next_new_line
    type Item = (&'a str, bool);

    fn next(&mut self) -> Option<Self::Item> {
        let (cur_end, new_line) = match self.memchr.next() {
            None => {
                if self.cur_start > self.string.len() {
                    return None;
                }
                (self.string.len(), false)
            }
            Some(end) => {
                // SAFETY: end never >= self.string.len()
                let ch = unsafe { *self.string.as_bytes().get_unchecked(end) };
                (end, ch == b';')
            }
        };
        // SAFETY: cur_end never > self.string.len()
        let s = unsafe { self.string.get_unchecked(self.cur_start..cur_end) };
        self.cur_start = cur_end + 1;
        Some((s, new_line))
    }
}

#[cfg(test)]
mod tests {
    use super::MappingSplitter;

    #[test]
    fn test_splitter() {
        let text =
      ";;yZCTnK,IAAO5F,gBAAkB,YACzB,IAAOC,YAAcC,UACrB;IAAOC,oBAAsB,YAE7B,EAAQ,QAER,EAAQ;;cAAe";

        let result = MappingSplitter::new(text)
            .map(|(s, n)| format!("[{}:{}]", s, n))
            .collect::<String>();
        insta::assert_snapshot!(result, @"[:true][:true][yZCTnK:false][IAAO5F:false][gBAAkB:false][YACzB:false][IAAOC:false][YAAcC:false][UACrB:true][IAAOC:false][oBAAsB:false][YAE7B:false][EAAQ:false][QAER:false][EAAQ:true][:true][cAAe:false]");
    }
}
