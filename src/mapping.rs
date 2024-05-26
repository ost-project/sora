use std::fmt::{Debug, Formatter};

/// `Position` represents a zero-based line and zero-based column in a file.
///
/// # Note
///
/// The source map specification does not define whether generated lines start at 0 or 1.
/// In this crate, both the line and column are 0-based.
/// However, it's important to consider that different implementations use different bases, for example:
///
/// - In almost all engine implementations, `Error.prototype.stack` and the source panel in devtools
///   have 1-based line and column.
/// - NPM libraries such as `sourcemap`, `acorn`, and `babel`, produce 1-based line and 0-based column.
/// - Tools like `esbuild` use 0-based line and column.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

impl Position {
    pub const fn max() -> Self {
        Self {
            line: u32::MAX,
            column: u32::MAX,
        }
    }

    pub const fn min() -> Self {
        Self { line: 0, column: 0 }
    }

    pub const fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

impl From<(u32, u32)> for Position {
    fn from((line, column): (u32, u32)) -> Self {
        Self::new(line, column)
    }
}

/// Presents a specific position in a specific source file.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SourceInfo {
    pub id: u32,
    pub position: Position,
}

impl SourceInfo {
    pub const fn new(id: u32, position: Position) -> Self {
        Self { id, position }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct OptionNum<const N: usize>([u32; N]);

impl<const N: usize> OptionNum<N> {
    const MISSING: u32 = 0xFFFFFFFF;

    #[inline]
    pub const fn missing() -> Self {
        Self([Self::MISSING; N])
    }

    #[inline]
    const fn new(v: [u32; N]) -> Self {
        Self(v)
    }

    #[inline]
    const fn get(&self) -> Option<&[u32; N]> {
        if self.is_missing() {
            None
        } else {
            Some(&self.0)
        }
    }

    #[inline]
    const fn is_missing(&self) -> bool {
        self.0[0] == Self::MISSING
    }
}

/// Presents an item of the `mappings`.
///
/// Lines and columns in `Mapping` are start at 0. See [Position].
#[derive(Clone, Eq, PartialEq)]
pub struct Mapping {
    generated: Position,

    // [source_id, source_line, source_col]
    source: OptionNum<3>,

    // [name_id]
    name: OptionNum<1>,
}

impl Debug for Mapping {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.generated.line, self.generated.column)?;
        if let Some(source_info) = self.source_info() {
            write!(
                f,
                " -> {}:{}:{}",
                source_info.id, source_info.position.line, source_info.position.column,
            )?;
            if let Some(name_id) = self.name_info() {
                write!(f, " ({})", name_id)?;
            }
        }
        Ok(())
    }
}

impl Mapping {
    #[inline(always)]
    pub const fn new(generated_line: u32, generated_col: u32) -> Self {
        Self {
            generated: Position {
                line: generated_line,
                column: generated_col,
            },
            source: OptionNum::missing(),
            name: OptionNum::missing(),
        }
    }

    #[inline(always)]
    pub const fn with_source(self, source_id: u32, source_line: u32, source_col: u32) -> Self {
        Self {
            source: OptionNum::new([source_id, source_line, source_col]),
            ..self
        }
    }

    #[inline(always)]
    pub const fn with_name(self, name_id: u32) -> Self {
        Self {
            name: OptionNum::new([name_id]),
            ..self
        }
    }
}

impl Mapping {
    /// Returns the generated position of the mapping.
    #[inline]
    pub fn generated(&self) -> Position {
        self.generated
    }

    /// Returns the source information if available.
    #[inline]
    pub fn source_info(&self) -> Option<SourceInfo> {
        self.source
            .get()
            .map(|&[source_id, source_line, source_col]| {
                SourceInfo::new(source_id, Position::new(source_line, source_col))
            })
    }

    /// Checks if the mapping has source information.
    #[inline]
    pub fn has_source(&self) -> bool {
        !self.source.is_missing()
    }

    /// Returns the name information if available.
    ///
    /// Note that in a mapping,
    /// name information will only be available if the source information is present.
    #[inline]
    pub fn name_info(&self) -> Option<u32> {
        self.name.get().map(|&[v]| v)
    }

    /// Checks if the mapping has name information.
    #[inline]
    pub fn has_name(&self) -> bool {
        !self.name.is_missing()
    }
}
