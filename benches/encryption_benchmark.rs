use criterion::{criterion_group, criterion_main, Criterion};
use crypt_lib::{
    util::config::load_config,
    util::encryption::{decrypt_file, encrypt_file},
};

// encrypt test with 850kb file
pub fn enc_benchmark(c: &mut Criterion) {
    let config = load_config().unwrap();

    c.bench_function("encrypt dracula", |b| {
        b.iter(|| encrypt_file(&config, "benches/files/dracula.txt", false))
    });
}

// encrypt test with 5mb file
pub fn enc_benchmark_large(c: &mut Criterion) {
    let config = load_config().unwrap();

    c.bench_function("encrypt dracula large file", |b| {
        b.iter(|| encrypt_file(&config, "benches/files/dracula-large.txt", false))
    });
}

// decrypt test with 850kb file
pub fn dec_benchmark(c: &mut Criterion) {
    let config = load_config().unwrap();

    c.bench_function("decrypt dracula", |b| {
        b.iter(|| decrypt_file(&config, "benches/files/dracula.crypt", None))
    });
}

// decrypt test with 5mb file
pub fn dec_benchmark_large(c: &mut Criterion) {
    let config = load_config().unwrap();

    c.bench_function("decrypt dracula large file", |b| {
        b.iter(|| decrypt_file(&config, "benches/files/dracula-large.crypt", None))
    });
}

pub fn cleanup(_c: &mut Criterion) {
    _ = std::fs::remove_file("benches/files/dracula.crypt");
    _ = std::fs::remove_file("benches/files/dracula-large.crypt");
    _ = std::fs::remove_file("benches/files/dracula-decrypted.txt");
    _ = std::fs::remove_file("benches/files/dracula-large-decrypted.txt");
}

criterion_group!(
    benches,
    enc_benchmark,
    enc_benchmark_large,
    dec_benchmark,
    dec_benchmark_large,
    cleanup
);
criterion_main!(benches);
