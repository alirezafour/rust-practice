use criterion::{Criterion, black_box, criterion_group, criterion_main};
use network_parse::model::Feed;

fn bench_feed_serialization(c: &mut Criterion) {
    // benchmark code here
}

criterion_group!(benches, bench_feed_serialization);
criterion_main!(benches);
