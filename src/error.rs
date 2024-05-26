pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unsupported source map format")]
    UnsupportedFormat,
    #[error("source map syntax error: {0}")]
    SyntaxError(#[from] simd_json::Error),
    #[error("a mapping is malformed")]
    MappingMalformed,
    #[error("a mapping overflows")]
    MappingOverflow,
    #[error("mappings are unordered")]
    UnorderedMappings,
    #[error("a mapping references unknown source #{0}")]
    UnknownSourceReference(u32),
    #[error("a mapping references unknown name #{0}")]
    UnknownNameReference(u32),

    #[error(
        "source map has {sources_len} sources but {sources_content_len} sourcesContent entries"
    )]
    InvalidSourcesContent {
        sources_len: u32,
        sources_content_len: u32,
    },

    #[cfg(feature = "extension")]
    #[error(
        "source map has {sources_len} sources but ignoreList[{idx}] references to {reference}"
    )]
    InvalidIgnoreList {
        sources_len: u32,
        idx: u32,
        reference: u32,
    },

    #[cfg(feature = "index-map")]
    #[error("section {section_id} error: {error}")]
    SectionError {
        section_id: u32,
        #[source]
        error: Box<Error>,
    },
}

impl Error {
    #[inline]
    pub(crate) fn invalid_sources_content(sources_len: u32, sources_content_len: u32) -> Self {
        Self::InvalidSourcesContent {
            sources_len,
            sources_content_len,
        }
    }

    #[inline]
    #[cfg(feature = "extension")]
    pub(crate) fn invalid_ignore_list(sources_len: u32, idx: u32, reference: u32) -> Self {
        Self::InvalidIgnoreList {
            sources_len,
            idx,
            reference,
        }
    }

    #[inline]
    #[cfg(feature = "index-map")]
    pub(crate) fn section_error(section_id: u32, error: Self) -> Self {
        Self::SectionError {
            section_id,
            error: Box::new(error),
        }
    }
}
