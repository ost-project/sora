#[derive(Debug, simd_json_derive::Deserialize)]
#[simd_json(rename_all = "camelCase")]
pub(crate) struct RawSourceMap<'a> {
    pub version: Option<u32>,
    pub file: Option<&'a str>,
    pub sources: Option<Vec<Option<&'a str>>>,
    pub source_root: Option<&'a str>,
    pub sources_content: Option<Vec<Option<&'a str>>>,
    pub names: Option<Vec<&'a str>>,
    pub mappings: Option<&'a str>,
    #[cfg(feature = "ignore_list")]
    pub ignore_list: Option<Vec<u32>>,
    #[cfg(feature = "index-map")]
    pub sections: Option<Vec<RawSection<'a>>>,
}

#[cfg(feature = "index-map")]
#[derive(Debug, simd_json_derive::Deserialize)]
pub(crate) struct RawSectionOffset {
    pub line: u32,
    pub column: u32,
}

#[cfg(feature = "index-map")]
#[derive(Debug, simd_json_derive::Deserialize)]
pub(crate) struct RawSection<'a> {
    pub offset: RawSectionOffset,
    // Note: referenced source maps are not supported
    // pub url: Option<&'a str>,
    pub map: Option<RawSourceMap<'a>>,
}
