use core::time::Duration;
use criterion::measurement::WallTime;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkGroup, Criterion};
use smallrand::*;

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = random_bytes, random_u32, random_u64
);
criterion_main!(benches);

pub fn random_bytes(c: &mut Criterion) {
    let mut g = c.benchmark_group("random_bytes");
    g.warm_up_time(Duration::from_millis(500));
    g.measurement_time(Duration::from_millis(1000));
    g.throughput(criterion::Throughput::Bytes(1024));

    fn bench(g: &mut BenchmarkGroup<WallTime>, name: &str, mut rng: impl Rng) {
        g.bench_function(name, |b| {
            let mut buf = [0u8; 1024];
            b.iter(|| {
                rng.fill_u8(&mut buf);
                black_box(buf);
            });
        });
    }

    let mut dev_urandom = DevUrandom::default();
    bench(&mut g, "xoshiro256++", Xoshiro256pp::from_entropy(&mut dev_urandom));
    bench(&mut g, "small", SmallRng::from_entropy(&mut dev_urandom));
    bench(&mut g, "chacha12", ChaCha12::from_entropy(&mut dev_urandom));
    bench(&mut g, "std", StdRng::from_entropy(&mut dev_urandom));

    g.finish()
}

pub fn random_u32(c: &mut Criterion) {
    let mut g = c.benchmark_group("random_u32");
    g.sample_size(1000);
    g.warm_up_time(Duration::from_millis(500));
    g.measurement_time(Duration::from_millis(1000));
    g.throughput(criterion::Throughput::Bytes(4));

    fn bench(g: &mut BenchmarkGroup<WallTime>, name: &str, mut rng: impl Rng) {
        g.bench_function(name, |b| {
            b.iter(|| rng.random::<u32>());
        });
    }

    let mut dev_urandom = DevUrandom::default();
    bench(&mut g, "xoshiro256++", Xoshiro256pp::from_entropy(&mut dev_urandom));
    bench(&mut g, "small", SmallRng::from_entropy(&mut dev_urandom));
    bench(&mut g, "chacha12", ChaCha12::from_entropy(&mut dev_urandom));
    bench(&mut g, "std", StdRng::from_entropy(&mut dev_urandom));

    g.finish()
}

pub fn random_u64(c: &mut Criterion) {
    let mut g = c.benchmark_group("random_u64");
    g.sample_size(1000);
    g.warm_up_time(Duration::from_millis(500));
    g.measurement_time(Duration::from_millis(1000));
    g.throughput(criterion::Throughput::Bytes(8));

    fn bench(g: &mut BenchmarkGroup<WallTime>, name: &str, mut rng: impl Rng) {
        g.bench_function(name, |b| {
            b.iter(|| rng.random::<u64>());
        });
    }

    let mut dev_urandom = DevUrandom::default();
    bench(&mut g, "xoshiro256++", Xoshiro256pp::from_entropy(&mut dev_urandom));
    bench(&mut g, "small", SmallRng::from_entropy(&mut dev_urandom));
    bench(&mut g, "chacha12", ChaCha12::from_entropy(&mut dev_urandom));
    bench(&mut g, "std", StdRng::from_entropy(&mut dev_urandom));

    g.finish()
}
