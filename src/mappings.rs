use crate::finder::{MappingFinder, MappingFinderImpl};
use crate::hint::{likely, unlikely};
use crate::mapping::{Mapping, Position};
use crate::splitter::MappingSplitter;
use crate::vlq::{VlqDecoder, VlqEncoder};
use crate::{ParseError, ParseResult, ValidateError, ValidateResult};
use std::io;
use std::io::Write;
use std::ops::Deref;

/// `Mappings` is a collection of [Mapping] entries.
#[derive(Debug, Clone, Default)]
pub struct Mappings(pub(crate) Vec<Mapping>);

impl Deref for Mappings {
    type Target = [Mapping];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "builder")]
impl Mappings {
    /// Creates a new `Mappings` from a vec of [Mapping] entries.
    ///
    /// The entries will be sorted by their generated positions to ensure data valid.
    pub fn new(raw: Vec<Mapping>) -> Self {
        let mut v = Self(raw);
        v.sort();
        v
    }

    /// Creates a new `Mappings` from a vec of [Mapping] entries without any check and sorting.
    ///
    /// # Safety
    ///
    /// This method does not sort the entries. The caller must ensure that the `Mapping` entries are
    /// in the correct order.
    /// Incorrect ordering may result in undefined behavior when finding in the mappings.
    pub unsafe fn new_unchecked(raw: Vec<Mapping>) -> Self {
        Self(raw)
    }
}

impl Mappings {
    /// Sorts mapping entries by their generated positions to ensure data valid.
    pub fn sort(&mut self) {
        self.0.sort_unstable_by_key(Mapping::generated)
    }

    /// Provides mutable access to the internal vec of [Mapping] entries.
    ///
    /// # Safety
    ///
    /// This method allows direct mutable access to the internal data structure.
    /// The caller must ensure that the data remains valid and properly ordered.
    /// Incorrect ordering may result in undefined behavior when finding in the mappings.
    pub unsafe fn inner_mut(&mut self) -> &mut Vec<Mapping> {
        &mut self.0
    }
}

impl Mappings {
    /// see [find_mapping](crate::BorrowedSourceMap::find_mapping).
    pub fn find_mapping<P>(&self, pos: P) -> Option<Mapping>
    where
        P: Into<Position>,
    {
        MappingFinderImpl::new(self).find(pos.into(), None)
    }

    /// see [find_mapping](crate::BorrowedSourceMap::finder).
    pub fn finder(&self) -> MappingFinder {
        MappingFinder::new(self)
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct ItemsCount {
    pub(crate) sources: u32,
    pub(crate) names: u32,
}

impl ItemsCount {
    pub fn new(sources: u32, names: u32) -> Self {
        Self { sources, names }
    }
}

impl Mappings {
    pub(crate) fn encode<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        let mut prev_generated_line = 0;
        let mut prev_generated_col = 0;
        let mut prev_source_id = 0;
        let mut prev_source_line = 0;
        let mut prev_source_col = 0;
        let mut prev_name_id = 0;

        for (idx, mapping) in self.0.iter().enumerate() {
            let generated_pos = mapping.generated();

            if generated_pos.line != prev_generated_line {
                prev_generated_col = 0;
                while generated_pos.line != prev_generated_line {
                    writer.write_all(&[b';'])?;
                    prev_generated_line += 1;
                }
            } else if idx != 0 {
                writer.write_all(&[b','])?;
            }

            let mut encoder = VlqEncoder::new(writer);

            encoder.encode(prev_generated_col, generated_pos.column)?;
            prev_generated_col = generated_pos.column;

            if let Some(source_info) = mapping.source_info() {
                encoder.encode(prev_source_id, source_info.id)?;
                prev_source_id = source_info.id;

                encoder.encode(prev_source_line, source_info.position.line)?;
                prev_source_line = source_info.position.line;

                encoder.encode(prev_source_col, source_info.position.column)?;
                prev_source_col = source_info.position.column;

                if let Some(name_id) = mapping.name_id() {
                    encoder.encode(prev_name_id, name_id)?;
                    prev_name_id = name_id;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn validate(&self, items_count: ItemsCount) -> ValidateResult<()> {
        // validate mappings
        // 1. generated pos is in order
        // 2. source_id has corresponding source
        // 3. name_id has corresponding name

        let mut last_generated_pos = Position::min();

        for mapping in &self.0 {
            let pos = mapping.generated();
            if pos.lt(&last_generated_pos) {
                return Err(ValidateError::MappingsUnordered);
            }
            last_generated_pos = pos;
            if let Some(source_info) = mapping.source_info() {
                if source_info.id >= items_count.sources {
                    return Err(ValidateError::UnknownSourceReference(source_info.id));
                }

                if let Some(name_id) = mapping.name_id() {
                    if name_id >= items_count.names {
                        return Err(ValidateError::UnknownNameReference(name_id));
                    }
                }
            }
        }

        Ok(())
    }
}

impl Mappings {
    pub(crate) fn empty() -> Self {
        Self(Vec::with_capacity(256))
    }
}

#[derive(Default)]
pub(crate) struct MappingsDecoder<'a> {
    source: &'a str,
    items_count: ItemsCount,
    #[cfg(feature = "index-map")]
    /// (generated_line, generated_col, source_id, name_id)
    state: (u32, u32, u32, u32),
}

impl<'a> MappingsDecoder<'a> {
    #[inline]
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            source,
            ..Default::default()
        }
    }

    #[inline]
    pub(crate) fn items_count(mut self, sources: u32, names: u32) -> Self {
        self.items_count = ItemsCount::new(sources, names);
        self
    }

    #[inline]
    #[cfg(feature = "index-map")]
    pub(crate) fn state(
        mut self,
        generated_line: u32,
        generated_col: u32,
        source_id: u32,
        name_id: u32,
    ) -> Self {
        self.state = (generated_line, generated_col, source_id, name_id);
        self
    }
}

impl<'a> MappingsDecoder<'a> {
    pub(crate) fn decode(&self) -> ParseResult<Mappings> {
        let mut mappings = Mappings::empty();
        self.decode_into(&mut mappings)?;
        Ok(mappings)
    }

    pub(crate) fn decode_into(&self, mappings: &mut Mappings) -> ParseResult<()> {
        let source = self.source;
        let items_count = self.items_count;

        let buffer = &mut mappings.0;

        #[cfg(feature = "index-map")]
        let (mut generated_line, mut generated_col, mut source_id, mut name_id) = self.state;
        #[cfg(not(feature = "index-map"))]
        let (mut generated_line, mut generated_col, mut source_id, mut name_id) = (0, 0, 0, 0);

        let mut source_line = 0;
        let mut source_col = 0;

        let mut decoder = VlqDecoder::new();

        let splitter = MappingSplitter::new(source);

        for (segment, next_new_line) in splitter {
            if likely!(!segment.is_empty()) {
                let nums = decoder.decode(segment)?;

                let mapping =
                    match nums.len() {
                        1 => {
                            if unlikely!(nums[0] < 0) {
                                return Err(ParseError::MappingsUnordered);
                            }
                            generated_col = (generated_col as i64 + nums[0]) as u32;
                            Mapping::new(generated_line, generated_col)
                        }
                        4 | 5 => {
                            if unlikely!(nums[0] < 0) {
                                return Err(ParseError::MappingsUnordered);
                            }
                            generated_col = (generated_col as i64 + nums[0]) as u32;

                            source_id = (source_id as i64 + nums[1]) as u32;
                            if unlikely!(source_id >= items_count.sources) {
                                return Err(ParseError::UnknownSourceReference(source_id));
                            }

                            source_line = (source_line as i64 + nums[2]) as u32;
                            source_col = (source_col as i64 + nums[3]) as u32;

                            let mut mapping = Mapping::new(generated_line, generated_col)
                                .with_source(source_id, source_line, source_col);

                            if nums.len() == 5 {
                                name_id = (name_id as i64 + nums[4]) as u32;
                                if unlikely!(name_id >= items_count.names) {
                                    return Err(ParseError::UnknownNameReference(name_id));
                                }
                                mapping = mapping.with_name(name_id)
                            }

                            mapping
                        }
                        _ => return Err(ParseError::MappingMalformed(segment.to_owned())),
                    };
                buffer.push(mapping);
            }

            if next_new_line {
                generated_line += 1;
                generated_col = 0;
            }
        }

        if let Some(mapping) = buffer.last() {
            if mapping.generated().line != generated_line {
                // There is only one scenario where the last mapping's line would differ from
                // the final generated_line: when the last line has no mappings.
                // Therefore, a mapping that points to the start of the last line
                // needs to be inserted to mark the end of the map.
                buffer.push(Mapping::new(generated_line, 0))
            }
        }

        Ok(())
    }
}
