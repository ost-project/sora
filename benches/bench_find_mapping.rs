use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;

macro_rules! assert_pos {
    (sora: $actual:expr, $expected:ident) => {
        assert_eq!(
            $actual.source_info().unwrap(),
            sora::SourceInfo::new($expected.0, ($expected.1, $expected.2).into())
        );
    };
    (sourcemap: $actual:expr, $expected:ident) => {
        assert_eq!(
            (
                $actual.get_src_id(),
                $actual.get_src_line(),
                $actual.get_src_col()
            ),
            $expected
        );
    };
}

fn benchmark_find_mapping(c: &mut Criterion) {
    let buf = fs::read("data/tsc.min.js.map").unwrap();

    let map_samples = &[
        ((340, 5636), (68, 619, 8)),
        ((340, 5649), (68, 620, 8)),
        ((340, 5673), (68, 625, 16)),
        ((340, 5676), (68, 626, 4)),
    ];

    {
        let mut bg = c.benchmark_group("find_mapping(one)");
        let &(pos, expected) = map_samples.first().unwrap();
        bg.bench_function("sora", |b| {
            let sm = sora::SourceMap::from(buf.clone()).unwrap();
            b.iter(|| {
                assert_pos!(sora: sm.find_mapping(pos).unwrap(), expected);
            })
        });
        bg.bench_function("sora(finder)", |b| {
            let sm = sora::SourceMap::from(buf.clone()).unwrap();
            b.iter(|| {
                assert_pos!(sora: sm.finder().find_mapping(pos).unwrap(), expected);
            })
        });
        bg.bench_function("sourcemap", |b| {
            let sm = sourcemap::SourceMap::from_slice(&buf).unwrap();

            b.iter(|| {
                let token = sm.lookup_token(pos.0, pos.1).unwrap();
                assert_pos!(sourcemap: token, expected);
            })
        });
    }
    {
        let mut bg = c.benchmark_group("find_mapping(sequential)");
        bg.bench_function("sora", |b| {
            let sm = sora::SourceMap::from(buf.clone()).unwrap();
            b.iter(|| {
                for &(pos, expected) in map_samples {
                    assert_pos!(sora: sm.find_mapping(pos).unwrap(), expected);
                }
            })
        });
        bg.bench_function("sora(finder)", |b| {
            let sm = sora::SourceMap::from(buf.clone()).unwrap();
            b.iter(|| {
                let finder = sm.finder();
                for &(pos, expected) in map_samples {
                    assert_pos!(sora: finder.find_mapping(pos).unwrap(), expected);
                }
            })
        });
        bg.bench_function("sourcemap", |b| {
            let sm = sourcemap::SourceMap::from_slice(&buf).unwrap();
            b.iter(|| {
                for &(pos, expected) in map_samples {
                    let token = sm.lookup_token(pos.0, pos.1).unwrap();
                    assert_pos!(sourcemap: token, expected);
                }
            })
        });
    }
}

criterion_group!(find_mapping, benchmark_find_mapping);
criterion_main!(find_mapping);
