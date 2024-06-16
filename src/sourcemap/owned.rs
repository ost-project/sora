use crate::sourcemap::BorrowedSourceMap;
use crate::Result;
use std::borrow::Cow;

/// `SourceMap` is a source map that owns all its internal strings,
/// providing a more straightforward and safe API for
/// users who do not need to manage the lifetimes of the strings manually.
pub type SourceMap = BorrowedSourceMap<'static>;

impl SourceMap {
    /// Creates a new owned [SourceMap] from a JSON buffer.
    #[inline]
    pub fn from(mut source: Vec<u8>) -> Result<Self> {
        Ok(BorrowedSourceMap::from_slice(&mut source)?.into_owned())
    }
}

impl BorrowedSourceMap<'_> {
    /// Convert a [BorrowedSourceMap] into a [SourceMap] that owns all its internal strings.
    pub fn into_owned(self) -> SourceMap {
        fn into_owned(i: Cow<'_, str>) -> Cow<'static, str> {
            Cow::Owned(i.into_owned())
        }

        let file = self.file.map(into_owned);

        let mappings = self.mappings;

        let names = {
            let mut vec = Vec::with_capacity(self.names.len());
            vec.extend(self.names.into_iter().map(into_owned));
            vec
        };

        let sources = {
            let mut vec = Vec::with_capacity(self.sources.len());
            vec.extend(self.sources.into_iter().map(|n| n.map(into_owned)));
            vec
        };

        let sources_content = {
            let mut vec = Vec::with_capacity(self.sources_content.len());
            vec.extend(self.sources_content.into_iter().map(|n| n.map(into_owned)));
            vec
        };

        #[cfg(feature = "extension")]
        let extension = self.extension;

        SourceMap {
            file,
            names,
            mappings,
            sources,
            sources_content,
            #[cfg(feature = "extension")]
            extension,
        }
    }
}
