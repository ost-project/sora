mod utils;

use sora::{BorrowedSourceMap, Error, Mapping, SourceMap};
use std::borrow::Cow;
use utils::read_file;

#[test]
fn test_parse() {
    assert!(matches!(
        SourceMap::from(b"".to_vec()),
        Err(Error::SyntaxError(..))
    ));

    assert!(matches!(
        SourceMap::from(b"{}".to_vec()),
        Err(Error::UnsupportedFormat)
    ));

    let sm = SourceMap::from(read_file("data/sum.js.map")).unwrap();
    {
        let sources = sm.sources();
        assert_eq!(sources[0], Some(Cow::Borrowed("../project/index.ts")));
        assert_eq!(sources[1], Some(Cow::Borrowed("../project/sum.ts")));
    }

    assert_eq!(sm.sources_content().len(), 2);

    {
        let mappings = sm.mappings();
        assert_eq!(mappings[0], Mapping::new(22, 0).with_source(0, 0, 0));
        assert_eq!(mappings[14], Mapping::new(32, 22).with_source(0, 2, 15));
    }

    for mut buf in [
        read_file("data/antd.min.js.map"),
        read_file("data/jquery.min.js.map"),
        read_file("data/tiny.js.map"),
        read_file("data/tsc.min.js.map"),
        #[cfg(feature = "index-map")]
        read_file("data/index-map.js.map"),
    ] {
        let owned_sm = SourceMap::from(buf.clone()).unwrap();
        let borrowed_sm = BorrowedSourceMap::from_slice(&mut buf).unwrap();

        assert_eq!(owned_sm.to_vec().unwrap(), borrowed_sm.to_vec().unwrap());
    }
}
