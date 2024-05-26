#![cfg(feature = "builder")]

use sora::{Error, Mapping, Mappings, SourceMap};
use std::borrow::Cow;

#[test]
fn test_sourcemap_builder() {
    let borrowed = SourceMap::builder()
        .with_file(Cow::Borrowed("test.file"))
        .with_sources(vec![Some(Cow::Borrowed("a.js"))])
        .with_sources_content(vec![None])
        .with_mappings(Mappings::new(vec![Mapping::new(0, 0).with_source(0, 1, 2)]))
        .build_borrowed()
        .unwrap()
        .to_string()
        .unwrap();
    insta::assert_snapshot!(borrowed, @r###"{"version":3,"file":"test.file","sources":["a.js"],"sourcesContent":[null],"mappings":"AACE"}"###);

    let owned = SourceMap::builder()
        .with_file(Cow::Borrowed("test.file"))
        .with_sources(vec![Some(Cow::Borrowed("a.js"))])
        .with_sources_content(vec![None])
        .with_mappings(Mappings::new(vec![Mapping::new(0, 0).with_source(0, 1, 2)]))
        .build()
        .unwrap()
        .to_string()
        .unwrap();
    insta::assert_snapshot!(owned, @r###"{"version":3,"file":"test.file","sources":["a.js"],"sourcesContent":[null],"mappings":"AACE"}"###);

    let err = SourceMap::builder()
        .with_file(Cow::Borrowed("test.file"))
        .with_sources(vec![Some(Cow::Borrowed("a.js"))])
        .with_sources_content(vec![None, None])
        .build();
    assert!(matches!(err, Err(Error::InvalidSourcesContent { .. })))
}
