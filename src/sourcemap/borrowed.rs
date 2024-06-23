use crate::finder::MappingFinder;
use crate::mapping::{Mapping, Position};
use crate::mappings::{ItemsCount, Mappings, MappingsDecoder};
use crate::sourcemap::raw::RawSourceMap;
use crate::{ParseError, ParseResult, ValidateError, ValidateResult};
use simd_json_derive::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::io;
use std::io::Write;
use std::iter::repeat_with;

/// `BorrowedSourceMap` is a source map containing borrowed or owned strings.
///
/// For a source map that owns all its internal strings, see [SourceMap](crate::SourceMap).
/// This struct can be converted into it using [Into::into].
///
/// # Methods
///
/// ## Parsing
///
/// You can create a `BorrowedSourceMap` using the following methods:
/// - [`BorrowedSourceMap::from_str`]
/// - [`BorrowedSourceMap::from_slice`]
/// - [`BorrowedSourceMap::from_slice_with_buffers`]
///
/// These methods take mutable references as parameters because they may modify
/// the data in place.
///
/// The parsing supports index maps if feature `index-map` enabled,
/// but sections will not be retained, and maps will be flattened as regular source maps.
///
/// ## Construction
///
/// When the `builder` feature is enabled, [SourceMapBuilder](crate::SourceMapBuilder) is
/// available to construct `BorrowedSourceMap` and [SourceMap](crate::SourceMap).
///
/// ## Access & Modification
///
/// The structure provides several methods to access and modify internal data, such as:
/// - [`sources`](BorrowedSourceMap::sources)
/// - [`sources_mut`](BorrowedSourceMap::sources_mut)
/// - unsafe [`sources_mut2`](BorrowedSourceMap::sources_mut2)
///
/// Unsafe methods allow for more extensive modifications to the source map.
///
/// Note: After making changes, call [`validate`](BorrowedSourceMap::validate) to ensure that
/// the source map remains valid and does not contain broken data.
///
/// ## Finding Mappings
///
/// To find mappings corresponding to specific positions, you can use:
/// - [`find_mapping`](BorrowedSourceMap::find_mapping)
/// - [`finder`](BorrowedSourceMap::finder)
///
/// ## Output
///
/// You can serialize the source map to json string using:
/// - [`write`](BorrowedSourceMap::write)
/// - [`to_vec`](BorrowedSourceMap::to_vec)
/// - [`to_string`](BorrowedSourceMap::to_string)
#[derive(Clone)]
pub struct BorrowedSourceMap<'a> {
    pub(crate) file: Option<Cow<'a, str>>,
    pub(crate) mappings: Mappings,
    pub(crate) names: Vec<Cow<'a, str>>,
    pub(crate) source_root: Option<Cow<'a, str>>,
    pub(crate) sources: Vec<Option<Cow<'a, str>>>,
    pub(crate) sources_content: Vec<Option<Cow<'a, str>>>,
    #[cfg(feature = "ignore_list")]
    pub(crate) ignore_list: Vec<u32>,
}

impl Debug for BorrowedSourceMap<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("SourceMap\n")?;
        f.write_str("  sources:\n")?;
        for (idx, source) in self.sources.iter().enumerate() {
            let source = source.as_ref().map(Cow::as_ref).unwrap_or("");
            writeln!(f, "    {idx}: {source}")?;
        }
        f.write_str("  names:\n")?;
        for (idx, name) in self.names.iter().enumerate() {
            writeln!(f, "    {idx}: {name}")?;
        }
        f.write_str("  mappings:\n")?;
        if !self.mappings.is_empty() {
            let mut last_mapping = &self.mappings[0];
            write!(f, "    {:?}", last_mapping)?;
            for mapping in self.mappings.iter().skip(1) {
                if mapping.generated().line != last_mapping.generated().line {
                    f.write_str("\n    ")?;
                } else {
                    f.write_str(", ")?;
                }
                write!(f, "{:?}", mapping)?;
                last_mapping = mapping;
            }
        }
        Ok(())
    }
}

impl<'a> BorrowedSourceMap<'a> {
    /// Finds the mapping for a given generated position.
    ///
    /// If an exact match is not found, this method returns the closest preceding mapping.
    /// If there are no preceding mappings, it returns `None`.
    ///
    /// # Example
    /// ```
    /// # use sora::{BorrowedSourceMap, Position};
    /// # let mut buf = r#"{"version": 3}"#.as_bytes().to_vec();
    /// let source_map = BorrowedSourceMap::from_slice(&mut buf).unwrap();
    /// if let Some(mapping) = source_map.find_mapping((1, 2)) {
    ///     println!("Mapping found: {:?}", mapping);
    /// } else {
    ///     println!("No mapping found for the given generated position.");
    /// }
    /// ```
    pub fn find_mapping<P>(&self, pos: P) -> Option<Mapping>
    where
        P: Into<Position>,
    {
        self.mappings.find_mapping(pos)
    }

    /// Creates a `MappingFinder` for the source map.
    ///
    /// This stateful finder is highly efficient for frequent mapping findings,
    /// especially when traversing the source map in small increments (e.g., sequentially
    /// finding mappings from start to finish in a source map for a minified file).
    ///
    /// # Example
    /// ```
    /// # use sora::{BorrowedSourceMap, Position};
    /// # let mut buf = r#"{"version": 3}"#.as_bytes().to_vec();
    /// let source_map = BorrowedSourceMap::from_slice(&mut buf).unwrap();
    /// let finder = source_map.finder();
    /// finder.find_mapping((1, 2));
    /// finder.find_mapping((1, 6));
    /// ```
    pub fn finder(&self) -> MappingFinder {
        self.mappings.finder()
    }

    /// Validates the source map.
    pub fn validate(&self) -> ValidateResult<()> {
        let sources_len = self.sources.len() as u32;
        let sources_content_len = self.sources_content.len() as u32;
        let names_len = self.names.len() as u32;

        if sources_content_len != sources_len {
            return Err(ValidateError::MismatchSourcesContent {
                sources_len,
                sources_content_len,
            });
        }

        // Note:
        // `ignore_list` is an additional feature that does not hinder the primary functionality
        // of source maps, so it is not subject to validation.

        self.mappings
            .validate(ItemsCount::new(sources_len, names_len))?;

        Ok(())
    }
}

impl<'a> BorrowedSourceMap<'a> {
    #[inline]
    pub fn file(&self) -> &Option<Cow<'a, str>> {
        &self.file
    }

    #[inline]
    pub fn file_mut(&mut self) -> &mut Option<Cow<'a, str>> {
        &mut self.file
    }

    #[inline]
    pub fn mappings(&self) -> &Mappings {
        &self.mappings
    }

    #[inline]
    pub fn mappings_mut(&mut self) -> &mut Mappings {
        &mut self.mappings
    }

    #[inline]
    pub fn names(&self) -> &[Cow<'a, str>] {
        &self.names
    }

    #[inline]
    pub fn names_mut(&mut self) -> &mut [Cow<'a, str>] {
        &mut self.names
    }

    #[inline]
    pub fn sources(&self) -> &[Option<Cow<'a, str>>] {
        &self.sources
    }

    #[inline]
    pub fn sources_mut(&mut self) -> &mut [Option<Cow<'a, str>>] {
        &mut self.sources
    }

    #[inline]
    pub fn sources_content(&self) -> &[Option<Cow<'a, str>>] {
        &self.sources_content
    }

    #[inline]
    pub fn sources_content_mut(&mut self) -> &mut [Option<Cow<'a, str>>] {
        &mut self.sources_content
    }

    #[inline]
    #[cfg(feature = "ignore_list")]
    pub fn ignore_list(&self) -> &[u32] {
        &self.ignore_list
    }

    /// This function directly returns &mut Vec and is not marked as unsafe
    /// because arbitrary modifications to the [ignore_list] will not compromise
    /// the primary functionality of source maps.
    #[inline]
    #[cfg(feature = "ignore_list")]
    pub fn ignore_list_mut(&mut self) -> &mut Vec<u32> {
        &mut self.ignore_list
    }
}

impl<'a> BorrowedSourceMap<'a> {
    /// Returns a mutable reference to the names.
    ///
    /// # Safety
    ///
    /// This function allows direct mutable access to the internal data structure.
    /// The caller must ensure that the name id referenced in the `mappings`
    /// cannot be greater than the length of the `names`.
    ///
    /// It's best to call [Self::validate] after making modifications.
    #[inline]
    pub unsafe fn names_mut2(&mut self) -> &mut Vec<Cow<'a, str>> {
        &mut self.names
    }

    /// Returns a mutable reference to the sources.
    ///
    /// # Safety
    ///
    /// This function allows direct mutable access to the internal data structure.
    /// The caller must ensure that the source id referenced in the `mappings`
    /// cannot be greater than the length of the `sources`.
    ///
    /// It's best to call [Self::validate] after making modifications.
    #[inline]
    pub unsafe fn sources_mut2(&mut self) -> &mut Vec<Option<Cow<'a, str>>> {
        &mut self.sources
    }

    /// Returns a mutable reference to the sources' content.
    ///
    /// # Safety
    ///
    /// This function allows direct mutable access to the internal data structure.
    /// The caller must ensure that the length of the `sources_content` matches the length of the `sources`.
    ///
    /// It's best to call [Self::validate] after making modifications.
    #[inline]
    pub unsafe fn sources_content_mut2(&mut self) -> &mut Vec<Option<Cow<'a, str>>> {
        &mut self.sources_content
    }
}

impl<'a> BorrowedSourceMap<'a> {
    fn from_raw(raw: RawSourceMap<'a>) -> ParseResult<Self> {
        if !matches!(raw.version, Some(3)) {
            return Err(ParseError::UnsupportedFormat);
        }
        #[cfg(feature = "index-map")]
        if let Some(sections) = raw.sections {
            return Self::process_index_map(sections);
        }

        Self::process_map(raw)
    }

    fn process_map(raw: RawSourceMap<'a>) -> ParseResult<Self> {
        let file = raw.file.map(Cow::Borrowed);

        let source_root = raw.source_root.map(Cow::Borrowed);

        let sources = raw
            .sources
            .map(|sources| Vec::from_iter(sources.into_iter().map(|s| s.map(Cow::Borrowed))))
            .unwrap_or_default();

        let sources_len = sources.len();

        let sources_content = if let Some(sources_content) = raw.sources_content {
            let sources_content_len = sources_content.len();
            if sources_content_len != sources_len {
                return Err(ParseError::MismatchSourcesContent {
                    sources_len: sources_len as u32,
                    sources_content_len: sources_content_len as u32,
                });
            }
            Vec::from_iter(sources_content.into_iter().map(|s| s.map(Cow::Borrowed)))
        } else {
            Vec::from_iter(repeat_with(|| None).take(sources_len))
        };

        let names = raw
            .names
            .map(|names| Vec::from_iter(names.into_iter().map(Cow::Borrowed)))
            .unwrap_or_default();

        let names_len = names.len();

        #[cfg(feature = "ignore_list")]
        let ignore_list = raw.ignore_list.unwrap_or_default();

        let mappings = MappingsDecoder::new(raw.mappings.unwrap_or_default())
            .items_count(sources_len as u32, names_len as u32)
            .decode()?;

        Ok(Self {
            file,
            source_root,
            sources,
            sources_content,
            names,
            mappings,
            #[cfg(feature = "ignore_list")]
            ignore_list,
        })
    }

    // To simplify the flattening logic of the index map, the following strategies are adopted:
    // 1. ignore the `file` attribute in all child maps,
    // 2. concat `source_root` for each `source`,
    // 3. merge `sources`/`names` from the child maps without performing any deduplication.
    #[cfg(feature = "index-map")]
    fn process_index_map(
        sections: Vec<crate::sourcemap::raw::RawSection<'a>>,
    ) -> ParseResult<Self> {
        let mut mappings = Mappings::empty();
        let mut names = vec![];
        let mut sources = vec![];
        let mut sources_content = vec![];

        #[cfg(feature = "ignore_list")]
        let mut ignore_list = vec![];

        let mut last_section_end_pos: Option<Position> = None;
        for section in sections.into_iter() {
            let current_section_start_pos = Position {
                line: section.offset.line,
                column: section.offset.column,
            };

            // offset should be greater than the last position of the last section
            if last_section_end_pos.is_some_and(|ref pos| current_section_start_pos.le(pos)) {
                return Err(ParseError::MappingsUnordered);
            }

            match section.map {
                Some(raw) => {
                    let start_names_id = names.len() as u32;
                    let start_sources_id = sources.len() as u32;

                    {
                        if let Some(raw_names) = raw.names {
                            names.extend(raw_names.into_iter().map(Cow::Borrowed));
                        }

                        if let Some(raw_sources) = raw.sources {
                            let raw_sources_len = raw_sources.len();

                            if let Some(raw_source_root) =
                                raw.source_root.filter(|sr| !sr.is_empty())
                            {
                                let source_root = raw_source_root.trim_end_matches('/');
                                sources.extend(raw_sources.into_iter().map(|s| {
                                    s.map(|source| {
                                        if !source.is_empty()
                                            && (source.starts_with('/')
                                                || source.starts_with("http:")
                                                || source.starts_with("https:"))
                                        {
                                            Cow::Borrowed(source)
                                        } else {
                                            Cow::Owned(format!("{}/{}", source_root, source))
                                        }
                                    })
                                }));
                            } else {
                                sources
                                    .extend(raw_sources.into_iter().map(|s| s.map(Cow::Borrowed)));
                            }

                            if let Some(raw_sources_content) = raw.sources_content {
                                let raw_sources_content_len = raw_sources_content.len();
                                if raw_sources_content_len != raw_sources_len {
                                    return Err(ParseError::MismatchSourcesContent {
                                        sources_len: raw_sources_len as u32,
                                        sources_content_len: raw_sources_content_len as u32,
                                    });
                                }
                                sources_content.extend(
                                    raw_sources_content
                                        .into_iter()
                                        .map(|s| s.map(Cow::Borrowed)),
                                );
                            } else {
                                sources_content.extend(repeat_with(|| None).take(raw_sources_len));
                            }
                        }
                    }

                    let end_sources_id = sources.len() as u32;
                    let end_names_id = names.len() as u32;

                    #[cfg(feature = "ignore_list")]
                    if let Some(raw_ignore_list) = raw.ignore_list {
                        if !raw_ignore_list.is_empty() {
                            for source_id in raw_ignore_list.into_iter() {
                                let fixed_source_id = source_id + start_sources_id;
                                if fixed_source_id >= end_sources_id {
                                    // skip if points to a non-existent source
                                    continue;
                                }
                                ignore_list.push(fixed_source_id);
                            }
                        }
                    }

                    MappingsDecoder::new(raw.mappings.unwrap_or_default())
                        .items_count(end_sources_id, end_names_id)
                        .state(
                            current_section_start_pos.line,
                            current_section_start_pos.column,
                            start_sources_id,
                            start_names_id,
                        )
                        .decode_into(&mut mappings)?;

                    last_section_end_pos = mappings.last().map(|m| m.generated());
                }
                None => {
                    // external maps referenced via URL are not supported,
                    // silently ignored without error.
                    last_section_end_pos = Some(current_section_start_pos)
                }
            }
        }

        Ok(Self {
            file: None,
            mappings,
            names,
            source_root: None,
            sources,
            sources_content,
            #[cfg(feature = "ignore_list")]
            ignore_list,
        })
    }
}

impl<'a> BorrowedSourceMap<'a> {
    /// Creates a new `BorrowedSourceMap` from a JSON buffer slice.
    ///
    /// The slice is mutable to facilitate in-place replacement of escape characters
    /// in the JSON string, allowing maximum data borrowing.
    #[inline]
    pub fn from_slice(json: &'a mut [u8]) -> ParseResult<Self> {
        Self::from_raw(RawSourceMap::from_slice(json)?)
    }

    /// Creates a new `BorrowedSourceMap` from a JSON string.
    ///
    /// The string is mutable to facilitate in-place replacement of escape characters
    /// in the JSON string, allowing maximum data borrowing.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(json: &'a mut str) -> ParseResult<Self> {
        Self::from_raw(RawSourceMap::from_str(json)?)
    }
}

impl BorrowedSourceMap<'_> {
    pub fn write<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        w.write_all(br#"{"version":3"#)?;

        if let Some(file) = self.file.as_deref() {
            w.write_all(br#","file":"#)?;
            file.json_write(w)?;
        }

        w.write_all(br#","sources":"#)?;
        self.sources.json_write(w)?;
        w.write_all(br#","sourcesContent":"#)?;
        self.sources_content.json_write(w)?;
        if !self.names.is_empty() {
            w.write_all(br#","names":"#)?;
            self.names.json_write(w)?;
        }

        w.write_all(br#","mappings":""#)?;
        self.mappings.encode(w)?;
        w.write_all(br#"""#)?;

        #[cfg(feature = "ignore_list")]
        if !self.ignore_list.is_empty() {
            w.write_all(br#","ignoreList":"#)?;
            self.ignore_list.json_write(w)?;
        }

        w.write_all(br#"}"#)
    }

    #[inline]
    pub fn to_vec(&self) -> io::Result<Vec<u8>> {
        let mut v = Vec::with_capacity(1024);
        self.write(&mut v)?;
        Ok(v)
    }

    #[inline]
    pub fn to_string(&self) -> io::Result<String> {
        self.to_vec()
            .map(|v| unsafe { String::from_utf8_unchecked(v) })
    }
}
