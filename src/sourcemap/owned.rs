use crate::sourcemap::BorrowedSourceMap;
use crate::Result;
use simd_json::Buffers;
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

/// `SourceMap` is a source map that owns all its internal strings,
/// providing a more straightforward and safe API for
/// users who do not need to manage the lifetimes of the strings manually.
#[derive(Clone)]
pub struct SourceMap(BorrowedSourceMap<'static>);

impl Debug for SourceMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl SourceMap {
    /// Creates a new `SourceMap` from JSON buffer.
    #[inline]
    pub fn from(mut source: Vec<u8>) -> Result<Self> {
        Ok(BorrowedSourceMap::from_slice(&mut source)?.into())
    }

    /// see [BorrowedSourceMap::from_slice].
    #[inline]
    pub fn from_slice(json: &mut [u8]) -> Result<Self> {
        Ok(BorrowedSourceMap::from_slice(json)?.into())
    }

    /// see [BorrowedSourceMap::from_slice_with_buffers].
    #[inline]
    pub fn from_slice_with_buffers(json: &mut [u8], buffers: &mut Buffers) -> Result<Self> {
        Ok(BorrowedSourceMap::from_slice_with_buffers(json, buffers)?.into())
    }

    /// see [BorrowedSourceMap::from_str].
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(json: &mut str) -> Result<Self> {
        Ok(BorrowedSourceMap::from_str(json)?.into())
    }
}

impl Deref for SourceMap {
    type Target = BorrowedSourceMap<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SourceMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<BorrowedSourceMap<'_>> for SourceMap {
    fn from(sm: BorrowedSourceMap<'_>) -> Self {
        fn into_owned(i: Cow<'_, str>) -> Cow<'static, str> {
            Cow::Owned(i.into_owned())
        }

        let file = sm.file.map(into_owned);

        let mappings = sm.mappings;

        let names = {
            let mut vec = Vec::with_capacity(sm.names.len());
            vec.extend(sm.names.into_iter().map(into_owned));
            vec
        };

        let sources = {
            let mut vec = Vec::with_capacity(sm.sources.len());
            vec.extend(sm.sources.into_iter().map(|n| n.map(into_owned)));
            vec
        };

        let sources_content = {
            let mut vec = Vec::with_capacity(sm.sources_content.len());
            vec.extend(sm.sources_content.into_iter().map(|n| n.map(into_owned)));
            vec
        };

        #[cfg(feature = "extension")]
        let extension = sm.extension;

        Self(BorrowedSourceMap {
            file,
            names,
            mappings,
            sources,
            sources_content,
            #[cfg(feature = "extension")]
            extension,
        })
    }
}

#[cfg(feature = "builder")]
mod builder {
    use crate::{BorrowedSourceMap, Mappings, Result, SourceMap};
    use std::borrow::Cow;

    impl SourceMap {
        pub fn builder() -> SourceMapBuilder<'static> {
            SourceMapBuilder::default()
        }
    }

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

        pub fn build_borrowed(self) -> Result<BorrowedSourceMap<'a>> {
            // SAFETY: just reuse code
            let v = unsafe { self.build_borrowed_unchecked() };
            v.validate()?;
            Ok(v)
        }

        /// Creates a new [BorrowedSourceMap] without validation.
        ///
        /// # Safety
        ///
        /// This function does not validate the values. The caller must ensure that
        /// the values are valid.
        pub unsafe fn build_borrowed_unchecked(self) -> BorrowedSourceMap<'a> {
            BorrowedSourceMap {
                file: self.file,
                mappings: self.mappings.unwrap_or_default(),
                names: self.names.unwrap_or_default(),
                sources: self.sources.unwrap_or_default(),
                sources_content: self.sources_content.unwrap_or_default(),
                #[cfg(feature = "extension")]
                extension: self.extension,
            }
        }

        pub fn build(self) -> Result<SourceMap> {
            self.build_borrowed().map(Into::into)
        }

        /// Creates a new [SourceMap] without validation.
        ///
        /// # Safety
        ///
        /// This function does not validate the values. The caller must ensure that
        /// the values are valid.
        pub unsafe fn build_unchecked(self) -> SourceMap {
            self.build_borrowed_unchecked().into()
        }
    }
}

#[cfg(feature = "builder")]
pub use builder::*;
