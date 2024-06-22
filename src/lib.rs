//! # sora
//!
//! This crate provides data structures and utilities for working with source maps in Rust.
//!
//! ## Getting Started
//!
//! ```ignore
//! use sora::SourceMap;
//!
//! // Load a source map from a buffer
//! let sm = SourceMap::from(buf).unwrap();
//!
//! // Find a mapping at the given position (line 10, column 12)
//! let found = sm.find_mapping((10, 12)).unwrap();
//!
//! // Print the found mapping
//! println!("Found mapping at (10, 12): {found}");
//! // Expected output: "Found mapping at (10, 12): 10:12 -> 1:6:8"
//! ```
//!
//! ## Overview
//!
//! ### `BorrowedSourceMap`
//!
//! [BorrowedSourceMap] is a source map containing borrowed or owned strings. It allows for efficient parsing and
//! manipulation of source maps, with several methods provided for creating, accessing, and modifying its contents.
//!
//! ### `SourceMap`
//!
//! [SourceMap] is a source map that owns all its internal strings,
//! providing a more straightforward and safe API for users who do not need
//! to manage the lifetimes of the strings manually.
//!
//! ### `Position`
//!
//! [Position] represents a 0-based line and 0-based column in a file.
//!
//! ### `Mapping`
//!
//! [Mapping] presents an item of the `mappings` in source maps.
//!
//! ## Features
//!
//! - `builder`: Enables [SourceMapBuilder] and functions like [Mappings::new] for manual construction of source maps.
//! - `index-map`: Enables support for index maps, as specified in [spec](https://tc39.es/source-map/#index-map).
//! - `extension`: Enables rarely-used source map features as defined in [spec](https://tc39.es/source-map), including `ignoreList`.
//!

mod error;
mod finder;
mod mapping;
mod mappings;
mod sourcemap;
mod splitter;
mod vlq;

pub use error::*;
pub use finder::*;
pub use mapping::*;
pub use mappings::*;
pub use sourcemap::*;
