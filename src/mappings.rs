use crate::finder::{MappingFinder, MappingFinderImpl};
use crate::mapping::{Mapping, Position};
use crate::splitter::Splitter;
use crate::vlq::{VlqDecoder, VlqEncoder};
use crate::{Error, Result};
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

#[derive(Debug, Copy, Clone)]
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

                if let Some(name_id) = mapping.name_info() {
                    encoder.encode(prev_name_id, name_id)?;
                    prev_name_id = name_id;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn validate(&self, items_count: ItemsCount) -> Result<()> {
        // validate mappings
        // 1. generated pos is in order
        // 2. source_id has corresponding source
        // 3. name_id has corresponding name

        let mut last_generated_pos = Position::min();

        for mapping in &self.0 {
            let pos = mapping.generated();
            if pos.lt(&last_generated_pos) {
                return Err(Error::UnorderedMappings);
            }
            last_generated_pos = pos;
            if let Some(source_info) = mapping.source_info() {
                if source_info.id >= items_count.sources {
                    return Err(Error::UnknownSourceReference(source_info.id));
                }

                if let Some(name_id) = mapping.name_info() {
                    if name_id >= items_count.names {
                        return Err(Error::UnknownNameReference(name_id));
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct DecodeState {
    pub(crate) generated_line: u32,
    pub(crate) generated_col: u32,
    pub(crate) source_id: u32,
    pub(crate) name_id: u32,
}

impl Mappings {
    pub(crate) fn decode(
        &mut self,
        source: &str,
        items_count: ItemsCount,
        state: &DecodeState,
    ) -> Result<()> {
        let DecodeState {
            mut generated_line,
            mut generated_col,
            mut source_id,
            mut name_id,
        } = *state;

        let mut source_line = 0;
        let mut source_col = 0;

        let mut decoder = VlqDecoder::new();

        // the ratio of source.len to mappings.len is generally between 5 and 7,
        // with most minified ones being > 6 and most unminified ones being < 6;
        // 6 is a conservative value here.
        // self.0.reserve(source.len() / 6);

        for line in Splitter::new(source, b';') {
            if !line.is_empty() {
                for segment in Splitter::new(line, b',') {
                    let nums = decoder.decode(segment)?;

                    match nums.len() {
                        1 => {
                            if nums[0] < 0 {
                                return Err(Error::UnorderedMappings);
                            }
                            generated_col = (generated_col as i64 + nums[0]) as u32;
                            self.0.push(Mapping::new(generated_line, generated_col));
                        }
                        4 | 5 => {
                            if nums[0] < 0 {
                                return Err(Error::UnorderedMappings);
                            }
                            generated_col = (generated_col as i64 + nums[0]) as u32;

                            source_id = (source_id as i64 + nums[1]) as u32;
                            if source_id >= items_count.sources {
                                return Err(Error::UnknownSourceReference(source_id));
                            }

                            source_line = (source_line as i64 + nums[2]) as u32;
                            source_col = (source_col as i64 + nums[3]) as u32;

                            let mut mapping = Mapping::new(generated_line, generated_col)
                                .with_source(source_id, source_line, source_col);

                            if nums.len() == 5 {
                                name_id = (name_id as i64 + nums[4]) as u32;
                                if name_id >= items_count.names {
                                    return Err(Error::UnknownNameReference(name_id));
                                }
                                mapping = mapping.with_name(name_id)
                            }

                            self.0.push(mapping);
                        }
                        _ => return Err(Error::MappingMalformed),
                    }
                }
            }

            generated_line += 1;
            generated_col = 0;
        }

        Ok(())
    }
}
