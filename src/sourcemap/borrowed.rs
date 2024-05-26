use crate::finder::MappingFinder;
use crate::mapping::{Mapping, Position};
use crate::mappings::{DecodeState, ItemsCount, Mappings};
use crate::sourcemap::raw::RawSourceMap;
use crate::{Error, Result};
use simd_json::Buffers;
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
    pub(crate) sources: Vec<Option<Cow<'a, str>>>,
    pub(crate) sources_content: Vec<Option<Cow<'a, str>>>,
    #[cfg(feature = "extension")]
    pub(crate) extension: crate::Extension,
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
    pub fn validate(&self) -> Result<()> {
        let sources_len = self.sources.len() as u32;
        let sources_content_len = self.sources_content.len() as u32;
        let names_len = self.names.len() as u32;

        if sources_content_len != sources_len {
            return Err(Error::invalid_sources_content(
                sources_len,
                sources_content_len,
            ));
        }
        #[cfg(feature = "extension")]
        self.extension.validate(sources_len)?;
        self.mappings
            .validate(ItemsCount::new(sources_len, names_len))
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
    #[cfg(feature = "extension")]
    pub fn extension(&self) -> &crate::Extension {
        &self.extension
    }

    #[inline]
    #[cfg(feature = "extension")]
    pub fn extension_mut(&mut self) -> &mut crate::Extension {
        &mut self.extension
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
    fn from_raw(raw: RawSourceMap<'a>) -> Result<Self> {
        if !matches!(raw.version, Some(3)) {
            return Err(Error::UnsupportedFormat);
        }

        let mut sm = Self::empty();
        #[cfg(feature = "index-map")]
        {
            if raw.sections.is_some() {
                sm.process_index_map(raw)?;
                return Ok(sm);
            }
        }
        sm.process_map(raw, Position::min())?;
        Ok(sm)
    }

    // use customize method `empty` rather than derive(Default)
    fn empty() -> Self {
        Self {
            file: None,
            mappings: Mappings::default(),
            names: Vec::new(),
            sources: Vec::new(),
            sources_content: Vec::new(),
            #[cfg(feature = "extension")]
            extension: crate::Extension::default(),
        }
    }

    #[inline]
    fn process_map(&mut self, raw: RawSourceMap<'a>, last_pos: Position) -> Result<()> {
        self.file = raw.file.map(Cow::Borrowed);

        let start_names_id = self.names.len() as u32;
        if let Some(names) = raw.names {
            let names_len = names.len();
            self.names.reserve(names_len);
            self.names.extend(names.into_iter().map(Cow::Borrowed));
        }

        let start_sources_id = self.sources.len() as u32;
        if let Some(sources) = raw.sources {
            let sources_len = sources.len();
            self.sources.reserve(sources_len);
            if let Some(source_root) = raw.source_root.filter(|sr| !sr.is_empty()) {
                let source_root = source_root.trim_end_matches('/');
                self.sources.extend(sources.into_iter().map(|s| {
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
                self.sources
                    .extend(sources.into_iter().map(|s| s.map(Cow::Borrowed)));
            }

            if let Some(sources_content) = raw.sources_content {
                let sources_content_len = sources_content.len();
                if sources_content_len != sources_len {
                    return Err(Error::invalid_sources_content(
                        sources_len as u32,
                        sources_content_len as u32,
                    ));
                }
                self.sources_content.reserve(sources_content_len);
                self.sources_content
                    .extend(sources_content.into_iter().map(|s| s.map(Cow::Borrowed)));
            } else {
                self.sources_content.reserve(sources_len);
                self.sources_content
                    .extend(repeat_with(|| None).take(sources_len));
            }
        }

        let end_sources_id = self.sources.len() as u32;
        let end_names_id = self.names.len() as u32;

        #[cfg(feature = "extension")]
        if let Some(ignore_list) = raw.ignore_list {
            if !ignore_list.is_empty() {
                self.extension.ignore_list.reserve(ignore_list.len());
                for (idx, source_id) in ignore_list.into_iter().enumerate() {
                    let fixed_source_id = source_id + start_sources_id;
                    if fixed_source_id >= end_sources_id {
                        return Err(Error::invalid_ignore_list(
                            end_sources_id,
                            idx as u32,
                            source_id,
                        ));
                    }
                    self.extension.ignore_list.push(fixed_source_id);
                }
            }
        }

        self.mappings.decode(
            raw.mappings.unwrap_or_default(),
            ItemsCount::new(end_sources_id, end_names_id),
            &DecodeState {
                generated_line: last_pos.line,
                generated_col: last_pos.column,
                source_id: start_sources_id,
                name_id: start_names_id,
            },
        )?;

        Ok(())
    }

    #[cold]
    #[cfg(feature = "index-map")]
    fn process_index_map(&mut self, raw: RawSourceMap<'a>) -> Result<()> {
        self.file = raw.file.map(Cow::Borrowed);

        let mut last_sec_end_pos = Position::min();
        for (section_id, section) in raw.sections.unwrap_or_default().into_iter().enumerate() {
            let section_id = section_id as u32;

            let sec_start_pos = Position {
                line: section.offset.line,
                column: section.offset.column,
            };

            // offset should be less than the last position of the last section
            if sec_start_pos.lt(&last_sec_end_pos) {
                return Err(Error::section_error(section_id, Error::UnorderedMappings));
            }

            match section.map {
                Some(map) => {
                    self.process_map(*map, sec_start_pos)
                        .map_err(|e| Error::section_error(section_id, e))?;
                    if let Some(mapping) = self.mappings.last() {
                        last_sec_end_pos = mapping.generated();
                    }
                }
                None => last_sec_end_pos = sec_start_pos,
            }
        }

        Ok(())
    }
}

impl<'a> BorrowedSourceMap<'a> {
    /// Creates a new `BorrowedSourceMap` from a JSON buffer slice.
    ///
    /// The slice is mutable to facilitate in-place replacement of escape characters
    /// in the JSON string, allowing maximum data borrowing.
    #[inline]
    pub fn from_slice(json: &'a mut [u8]) -> Result<Self> {
        Self::from_raw(RawSourceMap::from_slice(json)?)
    }

    /// Similar to [Self::from_slice],
    /// but reuses a buffer for strings to be copied in and out if needed.
    #[inline]
    pub fn from_slice_with_buffers(json: &'a mut [u8], buffers: &mut Buffers) -> Result<Self> {
        Self::from_raw(RawSourceMap::from_slice_with_buffers(json, buffers)?)
    }

    /// Creates a new `BorrowedSourceMap` from a JSON string.
    ///
    /// The string is mutable to facilitate in-place replacement of escape characters
    /// in the JSON string, allowing maximum data borrowing.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(json: &'a mut str) -> Result<Self> {
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

        #[cfg(feature = "extension")]
        if !self.extension.ignore_list.is_empty() {
            w.write_all(br#","ignoreList":"#)?;
            self.extension.ignore_list.json_write(w)?;
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
