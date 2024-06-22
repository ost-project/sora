use crate::{BorrowedSourceMap, Mappings, ValidateResult};
use std::borrow::Cow;

impl<'a> BorrowedSourceMap<'a> {
    pub fn builder() -> SourceMapBuilder<'a> {
        SourceMapBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct SourceMapBuilder<'a> {
    pub(crate) file: Option<Cow<'a, str>>,
    pub(crate) mappings: Option<Mappings>,
    pub(crate) names: Option<Vec<Cow<'a, str>>>,
    pub(crate) source_root: Option<Cow<'a, str>>,
    pub(crate) sources: Option<Vec<Option<Cow<'a, str>>>>,
    pub(crate) sources_content: Option<Vec<Option<Cow<'a, str>>>>,
    #[cfg(feature = "extension")]
    pub(crate) extension: crate::Extension,
}

impl<'a> SourceMapBuilder<'a> {
    #[inline(always)]
    pub fn with_file(mut self, file: Cow<'a, str>) -> Self {
        self.file = Some(file);
        self
    }

    #[inline(always)]
    pub fn with_mappings(mut self, mappings: Mappings) -> Self {
        self.mappings = Some(mappings);
        self
    }

    #[inline(always)]
    pub fn with_names(mut self, names: Vec<Cow<'a, str>>) -> Self {
        self.names = Some(names);
        self
    }

    #[inline(always)]
    pub fn with_source_root(mut self, source_root: Cow<'a, str>) -> Self {
        self.source_root = Some(source_root);
        self
    }

    #[inline(always)]
    pub fn with_sources(mut self, sources: Vec<Option<Cow<'a, str>>>) -> Self {
        self.sources = Some(sources);
        self
    }

    #[inline(always)]
    pub fn with_sources_content(mut self, sources_content: Vec<Option<Cow<'a, str>>>) -> Self {
        self.sources_content = Some(sources_content);
        self
    }

    #[cfg(feature = "extension")]
    #[inline(always)]
    pub fn with_extension(mut self, extension: crate::Extension) -> Self {
        self.extension = extension;
        self
    }

    pub fn build(self) -> ValidateResult<BorrowedSourceMap<'a>> {
        // SAFETY: just reuse code
        let v = unsafe { self.build_unchecked() };
        v.validate()?;
        Ok(v)
    }

    /// Creates a new [BorrowedSourceMap] without validation.
    ///
    /// # Safety
    ///
    /// This function does not validate the values. The caller must ensure that
    /// the values are valid.
    pub unsafe fn build_unchecked(self) -> BorrowedSourceMap<'a> {
        BorrowedSourceMap {
            file: self.file,
            mappings: self.mappings.unwrap_or_default(),
            names: self.names.unwrap_or_default(),
            source_root: self.source_root,
            sources: self.sources.unwrap_or_default(),
            sources_content: self.sources_content.unwrap_or_default(),
            #[cfg(feature = "extension")]
            extension: self.extension,
        }
    }
}
