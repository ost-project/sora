use std::error::Error;

pub type ParseResult<T> = Result<T, ParseError>;
pub type ValidateResult<T> = Result<T, ValidateError>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ParseError {
    #[error("unsupported source map format")]
    UnsupportedFormat,
    #[error("source map syntax error: {0}")]
    Syntax(Box<dyn Error>),
    #[error("a mapping is malformed: \"{0}\"")]
    MappingMalformed(String),
    #[error("mappings are unordered")]
    MappingsUnordered,
    #[error("a mapping references unknown source #{0}")]
    UnknownSourceReference(u32),
    #[error("a mapping references unknown name #{0}")]
    UnknownNameReference(u32),
    #[error(
        "source map has {} sources but {} sourcesContent entries",
        sources_len,
        sources_content_len
    )]
    MismatchSourcesContent {
        sources_len: u32,
        sources_content_len: u32,
    },
}

impl From<simd_json::Error> for ParseError {
    fn from(value: simd_json::Error) -> Self {
        Self::Syntax(Box::new(value))
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateError {
    #[error("mappings are unordered")]
    MappingsUnordered,
    #[error("a mapping references unknown source #{0}")]
    UnknownSourceReference(u32),
    #[error("a mapping references unknown name #{0}")]
    UnknownNameReference(u32),
    #[error(
        "source map has {} sources but {} sourcesContent entries",
        sources_len,
        sources_content_len
    )]
    MismatchSourcesContent {
        sources_len: u32,
        sources_content_len: u32,
    },
}
