use crate::sourcemap::BorrowedSourceMap;
use crate::ParseResult;
use std::borrow::Cow;

/// `SourceMap` is a type alias to [BorrowedSourceMap] but owns all its internal strings,
/// providing a more straightforward and safe API for
/// users who do not need to manage the lifetimes of the strings manually.
pub type SourceMap = BorrowedSourceMap<'static>;

impl SourceMap {
    /// Creates a new owned [SourceMap] from a JSON buffer.
    #[inline]
    pub fn from(mut source: Vec<u8>) -> ParseResult<Self> {
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

        let names = self.names.into_iter().map(into_owned).collect();

        let source_root = self.source_root.map(into_owned);

        let sources = self
            .sources
            .into_iter()
            .map(|n| n.map(into_owned))
            .collect();

        let sources_content = self
            .sources_content
            .into_iter()
            .map(|n| n.map(into_owned))
            .collect();

        #[cfg(feature = "ignore_list")]
        let ignore_list = self.ignore_list;

        SourceMap {
            file,
            names,
            mappings,
            source_root,
            sources,
            sources_content,
            #[cfg(feature = "ignore_list")]
            ignore_list,
        }
    }
}
