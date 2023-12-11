use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion};
use crypt_lib::{
    filespawn::file_generator::{generate_files, SAVE_PATH},
    util::{encryption::{compute_hash, decrypt_file, encrypt_file, generate_uuid, FileCrypt, self}, common::read_to_vec_u8},
    util::{config::load_config, path::walk_directory},
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

// encrypt test with 850kb file
pub fn bench_just_enc(c: &mut Criterion) {

    // minumum setup needed to use encryption function
    let s = String::from("");
    let pb = PathBuf::new();
    let b: [u8;32] = [0u8;32];
    let mut fc = FileCrypt::new(s.clone(),s,pb,b);
    let contents = read_to_vec_u8(DRACULA_NORMAL);

    c.bench_function("encrypt contents of dracula", |b| {
        b.iter(|| encryption::encryption(&mut fc, &contents))
    });
}

// encrypt test with 5mb file
pub fn enc_benchmark_large(c: &mut Criterion) {
    let mut config = load_config().unwrap();
    config.retain = true;

    c.bench_function("encrypt dracula large", |b| {
        b.iter(|| encrypt_file(&config, DRACULA_LARGE, false))
    });
}

// encrypt test with 850kb file
pub fn enc_many_files_benchmark(c: &mut Criterion) {
    let mut config: crypt_lib::util::config::Config = load_config().unwrap();
    config.retain = true;

    // c.sample_size(10);

    _ = generate_files();
    // get vec of dir
    let dir = walk_directory(SAVE_PATH, &config).expect("could not find directory!");

    c.bench_function("encrypt 100 random files", |b| {
        b.iter(|| {
            for path in &dir {
                encrypt_file(&config, path.display().to_string().as_str(), false)
            }
        })
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
    c.bench_function("generate 26 digit uuid", |b| b.iter(|| generate_uuid()));
}

pub fn cleanup(_c: &mut Criterion) {
    _ = std::fs::remove_file(DRACULA_CRYPT);
    _ = std::fs::remove_file(DRACULA_LCRYPT);
    _ = std::fs::remove_file(DRACULA_DECRYPT);
    _ = std::fs::remove_file(DRACULA_LDECRYPT);
    _ = std::fs::remove_dir(SAVE_PATH);
}

criterion_group!(
    benches,
    enc_benchmark,
    bench_just_enc,
    enc_benchmark_large,
    enc_many_files_benchmark,
    dec_benchmark,
    dec_benchmark_large,
    test_compute_hash,
    test_generate_uuid,
    cleanup
);
criterion_main!(benches);
