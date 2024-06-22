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
}

#[cfg(test)]
mod tests {
    use super::RawSourceMap;
    use simd_json_derive::Deserialize;

    #[test]
    fn test_parse_success() {
        let mut bytes = br#"{
    "version":3,
    "file":"sum.js",
    "sources":["sum.ts"],
    "names":[],
    "mappings":";;;AAAO,IAAM,GAAG,GAAG,UAAC,CAAS,EAAE,CAAS,IAAK,OAAA,CAAC,GAAG,CAAC,EAAL,CAAK,CAAA;AAArC,QAAA,GAAG,OAAkC"
}"#.to_vec();
        RawSourceMap::from_slice(bytes.as_mut_slice()).unwrap();
    }

    #[test]
    fn test_parse_error() {
        let mut bytes = br#"{
    "version":3,
    "file":"sum.js",
    "sources":["sum.ts"],
    "names":[]
    "mappings":";;;AAAO,IAAM,GAAG,GAAG,UAAC,CAAS,EAAE,CAAS,IAAK,OAAA,CAAC,GAAG,CAAC,EAAL,CAAK,CAAA;AAArC,QAAA,GAAG,OAAkC"
}"#.to_vec();
        assert!(RawSourceMap::from_slice(bytes.as_mut_slice()).is_err())
    }
}
