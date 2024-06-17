use crate::{Error, Result};

/// Represents rarely-used source map features defined in <https://tc39.es/source-map>
///
/// - `ignoreList`: <https://tc39.es/source-map/#ignorelist>
///
#[derive(Debug, Default, Clone)]
pub struct Extension {
    pub(crate) ignore_list: Vec<u32>,
}

impl Extension {
    pub fn ignore_list(&self) -> &[u32] {
        &self.ignore_list
    }

    pub fn ignore_list_mut(&mut self) -> &mut Vec<u32> {
        &mut self.ignore_list
    }
}

impl Extension {
    pub(crate) fn from_raw(ignore_list: Option<Vec<u32>>) -> Self {
        let ignore_list = ignore_list.unwrap_or_default();
        Self { ignore_list }
    }

    pub(crate) fn validate(&self, sources_count: u32) -> Result<()> {
        if let Some((idx, &id)) = self
            .ignore_list
            .iter()
            .enumerate()
            .find(|&(_, &id)| id >= sources_count)
        {
            return Err(Error::invalid_ignore_list(sources_count, idx as u32, id));
        }

        Ok(())
    }
}

#[cfg(feature = "builder")]
mod builder {
    use super::Extension;

    impl Extension {
        pub fn builder() -> ExtensionBuilder {
            ExtensionBuilder::default()
        }
    }

    #[derive(Debug, Default)]
    pub struct ExtensionBuilder {
        ignore_list: Vec<u32>,
    }

    #[allow(clippy::needless_update)]
    impl ExtensionBuilder {
        #[inline(always)]
        pub fn with_ignore_list(self, ignore_list: Vec<u32>) -> Self {
            Self {
                ignore_list,
                ..self
            }
        }

        #[inline(always)]
        pub fn build(self) -> Extension {
            Extension {
                ignore_list: self.ignore_list,
            }
        }
    }
}
#[cfg(feature = "builder")]
pub use builder::*;
