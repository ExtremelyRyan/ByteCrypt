use criterion::{criterion_group, criterion_main, Criterion};
use crypt_lib::{
    util::config::load_config,
    util::encryption::{decrypt_file, encrypt_file, compute_hash, generate_uuid},
};

#[cfg(target_os = "linux")]
static DRACULA_NORMAL: &str = "benches/files/dracula.txt";
#[cfg(target_os = "linux")]
static DRACULA_LARGE: &str = "benches/files/dracula-large.txt";
#[cfg(target_os = "linux")]
static DRACULA_CRYPT: &str = "benches/files/dracula.crypt";
#[cfg(target_os = "linux")]
static DRACULA_LCRYPT: &str = "benches/files/dracula-large.crypt";
#[cfg(target_os = "linux")]
static DRACULA_DECRYPT: &str = "benches/files/dracula-decrypted.txt";
#[cfg(target_os = "linux")]
static DRACULA_LDECRYPT: &str = "benches/files/dracula-large-decrypted.txt";

#[cfg(target_os = "windows")] 
static DRACULA_NORMAL: &str = "benches\\files\\dracula.txt";
#[cfg(target_os = "windows")] 
static DRACULA_LARGE: &str = "benches\\files\\dracula-large.txt";
#[cfg(target_os = "windows")] 
static DRACULA_CRYPT: &str = "benches\\files\\dracula.crypt";
#[cfg(target_os = "windows")] 
static DRACULA_LCRYPT: &str = "benches\\files\\dracula-large.crypt";
#[cfg(target_os = "windows")]
static DRACULA_DECRYPT: &str = "benches\\files\\dracula-decrypted.txt";
#[cfg(target_os = "windows")]
static DRACULA_LDECRYPT: &str = "benches\\files\\dracula-large-decrypted.txt";



// encrypt test with 850kb file
pub fn enc_benchmark(c: &mut Criterion) {
    let mut config = load_config().unwrap();
    config.retain = true;

    c.bench_function("encrypt dracula", |b| {
        b.iter(|| encrypt_file(&config, DRACULA_NORMAL, false))
    });
}

// encrypt test with 5mb file
pub fn enc_benchmark_large(c: &mut Criterion) {
    let mut config = load_config().unwrap();
    config.retain = true;

    c.bench_function("encrypt dracula large file", |b| {
        b.iter(|| encrypt_file(&config, DRACULA_LARGE, false))
    });
}

// decrypt test with 850kb file
pub fn dec_benchmark(c: &mut Criterion) {
    let mut config = load_config().unwrap();
    config.retain = true;

    c.bench_function("decrypt dracula", |b| {
        b.iter(|| decrypt_file(&config, DRACULA_CRYPT, None))
    });
}

// decrypt test with 5mb file
pub fn dec_benchmark_large(c: &mut Criterion) {
    let mut config = load_config().unwrap();
    config.retain = true;

    c.bench_function("decrypt dracula large file", |b| {
        b.iter(|| decrypt_file(&config, DRACULA_LCRYPT, None))
    });
}

// test generating a hash
pub fn test_compute_hash(c: &mut Criterion) {
    let contents: Vec<u8> = std::fs::read(DRACULA_NORMAL).unwrap();

    c.bench_function("computing 32-bit hash", |b| {
        b.iter(|| compute_hash(&contents))
    });
}

// test generation of a 26 digit uuid
pub fn test_generate_uuid(c: &mut Criterion) {
    c.bench_function("generate 26 digit uuid", |b| {
        b.iter(|| generate_uuid())
    });
}

pub fn cleanup(_c: &mut Criterion) {
    _ = std::fs::remove_file(DRACULA_CRYPT);
    _ = std::fs::remove_file(DRACULA_LCRYPT);
    _ = std::fs::remove_file(DRACULA_DECRYPT);
    _ = std::fs::remove_file(DRACULA_LDECRYPT);
}

criterion_group!(
    benches,
    enc_benchmark,
    enc_benchmark_large,
    dec_benchmark,
    dec_benchmark_large,
    test_compute_hash,
    test_generate_uuid,
    cleanup
);
criterion_main!(benches);
