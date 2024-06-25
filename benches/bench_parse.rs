use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use std::fs;

// Parse is a benchmark that is highly affected by memory allocation performance.
// To reduce the impact, mimalloc is used as the allocator,
// so `owned` and `sourcemap` will be faster than using the default allocator.
use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn sora_parse_borrowed(mut data: Vec<u8>) {
    black_box(sora::BorrowedSourceMap::from_slice(&mut data).unwrap());
}

fn sora_parse_owned(data: Vec<u8>) {
    black_box(sora::SourceMap::from(data).unwrap());
}

fn sentry_parse(data: Vec<u8>) {
    black_box(sentry_sourcemap::SourceMap::from_slice(&data).unwrap());
}

#[cfg(feature = "index-map")]
fn sentry_parse_index(data: Vec<u8>) {
    black_box(sentry_sourcemap::SourceMapIndex::from_slice(&data).unwrap());
}

#[cfg(feature = "index-map")]
fn sentry_parse_index_flatten(data: Vec<u8>) {
    black_box(
        sentry_sourcemap::SourceMapIndex::from_slice(&data)
            .unwrap()
            .flatten()
            .unwrap(),
    );
}

fn oxc_parse(data: Vec<u8>) {
    let data = unsafe { String::from_utf8_unchecked(data) };
    black_box(oxc_sourcemap::SourceMap::from_json_string(&data).unwrap());
}

fn benchmark_parse(c: &mut Criterion) {
    #[rustfmt::skip]
    let cases = [
        ("tiny", fs::read("benches/data/tiny.js.map").unwrap(), BatchSize::SmallInput),
        ("jquery", fs::read("benches/data/jquery.min.js.map").unwrap(), BatchSize::SmallInput),
        ("antd", fs::read("benches/data/antd.min.js.map").unwrap(), BatchSize::LargeInput),
        ("tsc", fs::read("benches/data/tsc.min.js.map").unwrap(), BatchSize::LargeInput)
    ];
    for (name, buf, batch_size) in cases {
        let mut bg = c.benchmark_group(format!("parse({name})"));
        bg.bench_with_input("sora(borrowed)", &buf, |b, input| {
            b.iter_batched(|| input.clone(), sora_parse_borrowed, batch_size)
        });
        bg.bench_with_input("sora(owned)", &buf, |b, input| {
            b.iter_batched(|| input.clone(), sora_parse_owned, batch_size)
        });
        bg.bench_with_input("sentry", &buf, |b, input| {
            b.iter_batched(|| input.clone(), sentry_parse, batch_size)
        });
        bg.bench_with_input("oxc", &buf, |b, input| {
            b.iter_batched(|| input.clone(), oxc_parse, batch_size)
        });
    }

    #[cfg(feature = "index-map")]
    {
        #[rustfmt::skip]
        let cases = [
            ("index-map", fs::read("benches/data/index-map.js.map").unwrap(), BatchSize::SmallInput),
        ];
        for (name, buf, batch_size) in cases {
            let mut bg = c.benchmark_group(format!("parse({name})"));
            bg.bench_with_input("sora(borrowed)", &buf, |b, input| {
                b.iter_batched(|| input.clone(), sora_parse_borrowed, batch_size)
            });
            bg.bench_with_input("sora(owned)", &buf, |b, input| {
                b.iter_batched(|| input.clone(), sora_parse_owned, batch_size)
            });
            bg.bench_with_input("sentry", &buf, |b, input| {
                b.iter_batched(|| input.clone(), sentry_parse_index, batch_size)
            });
            bg.bench_with_input("sentry(flatten)", &buf, |b, input| {
                b.iter_batched(|| input.clone(), sentry_parse_index_flatten, batch_size)
            });
        }
    }
}

criterion_group!(parse, benchmark_parse);
criterion_main!(parse);
