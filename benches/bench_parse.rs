mod utils;

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use utils::*;

fn parse_sora_borrowed(mut data: Vec<u8>) {
    black_box(sora::BorrowedSourceMap::from_slice(&mut data).unwrap());
}

fn parse_sora_owned(data: Vec<u8>) {
    black_box(sora::SourceMap::from(data).unwrap());
}

fn parse_sourcemap(data: Vec<u8>) {
    black_box(sourcemap::SourceMap::from_slice(&data).unwrap());
}

fn benchmark_parse(c: &mut Criterion) {
    for (name, buf) in [
        ("antd", read_file("data/antd.min.js.map")),
        ("jquery", read_file("data/jquery.min.js.map")),
        ("tiny", read_file("data/tiny.js.map")),
        ("tsc", read_file("data/tsc.min.js.map")),
    ] {
        let mut bg = c.benchmark_group(format!("parse({name})"));
        bg.bench_with_input("sora(borrowed)", &buf, |b, input| {
            b.iter_batched(|| input.clone(), parse_sora_borrowed, BatchSize::SmallInput)
        });
        bg.bench_with_input("sora(owned)", &buf, |b, input| {
            b.iter_batched(|| input.clone(), parse_sora_owned, BatchSize::SmallInput)
        });
        bg.bench_with_input("sourcemap", &buf, |b, input| {
            b.iter_batched(|| input.clone(), parse_sourcemap, BatchSize::SmallInput)
        });
    }
}

criterion_group!(parse, benchmark_parse);
criterion_main!(parse);
