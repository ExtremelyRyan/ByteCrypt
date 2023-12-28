use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion};
use crypt_core::{
    common::get_file_bytes,
    filecrypt::{encrypt_file, generate_uuid, FileCrypt},
    *,
};

#[cfg(target_os = "linux")]
static DRACULA: &str = "benches/files/dracula.txt";
#[cfg(target_os = "linux")]
static SHAKESPEARE: &str = "benches/files/Shakespeare.txt";
#[cfg(target_os = "linux")]
static DRACULA_CRYPT: &str = "benches/files/dracula.crypt";
#[cfg(target_os = "linux")]
static DRACULA_DECRYPT: &str = "benches/files/dracula-decrypted.txt";
#[cfg(target_os = "linux")]
static SHAKESPEARE_CRYPT: &str = "benches/files/Shakespeare.crypt";
#[cfg(target_os = "linux")]
static SHAKESPEARE_DECRYPT: &str = "benches/files/Shakespeare-decrypted.txt";

#[cfg(target_os = "windows")]
static DRACULA: &str = "benches\\files\\dracula.txt";
#[cfg(target_os = "windows")]
static SHAKESPEARE: &str = "benches\\files\\Shakespeare.txt";
#[cfg(target_os = "windows")]
static DRACULA_CRYPT: &str = "benches\\files\\dracula.crypt";
#[cfg(target_os = "windows")]
static SHAKESPEARE_CRYPT: &str = "benches\\files\\Shakespeare.crypt";
#[cfg(target_os = "windows")]
static DRACULA_DECRYPT: &str = "benches\\files\\dracula-decrypted.txt";
#[cfg(target_os = "windows")]
static SHAKESPEARE_DECRYPT: &str = "benches\\files\\Shakespeare-decrypted.txt";

// encrypt test with 850kb file
pub fn enc_benchmark(c: &mut Criterion) {
    c.bench_function("full file encryption (dracula.txt)", |b| {
        b.iter(|| encrypt_file(DRACULA, false))
    });
}

// encrypt test with 850kb file
pub fn dracula_content_encryption(c: &mut Criterion) {
    // minumum setup needed to use encryption function
    let s = String::from("");
    let pb = PathBuf::new();
    let b: [u8; 32] = [0u8; 32];
    let mut fc = FileCrypt::new(s.clone(), s, "".to_string(), pb, b);
    let contents = get_file_bytes(DRACULA);

    c.bench_function("encrypt contents of dracula", |b| {
        b.iter(|| encryption::encrypt(&mut fc, &contents))
    });
}

// encrypt test with 5mb file
pub fn shakespeare_content_encryption(c: &mut Criterion) {
    // minumum setup needed to use encryption function
    let s = String::from("");
    let pb = PathBuf::new();
    let b: [u8; 32] = [0u8; 32];
    let mut fc = FileCrypt::new(s.clone(), s, "".to_string(), pb, b);
    let contents = get_file_bytes(SHAKESPEARE);

    c.bench_function("encrypt contents of shakespeare", |b| {
        b.iter(|| encryption::encrypt(&mut fc, &contents))
    });
}

// encrypt test with 5mb file
pub fn enc_benchmark_large(c: &mut Criterion) {
    c.bench_function("full file encryption (shakespeare)", |b| {
        b.iter(|| encrypt_file(SHAKESPEARE, false))
    });
}

// encrypt test with 850kb file
// pub fn enc_many_files_benchmark(c: &mut Criterion) {
//     {
//         let mut config = get_config_write();
//         config.retain = true;
//     }

//     // c.sample_size(10);

//     _ = generate_files();
//     // get vec of dir
//     let dir = walk_directory(SAVE_PATH).expect("could not find directory!");

//     let mut group = c.benchmark_group("encrypt 10 random files 10 times");
//     group.sample_size(500);
//     group.bench_function("encrypt 100 random files", |c| {
//         c.iter(|| {
//             for path in &dir {
//                 encrypt_file(path.display().to_string().as_str(), false)
//             }
//         })
//     });
//     group.finish();
// }

// decrypt test with 850kb file
pub fn dec_benchmark(c: &mut Criterion) {
    c.bench_function("decrypt dracula", |b| {
        b.iter(|| crate::filecrypt::decrypt_file(DRACULA_CRYPT, None))
    });
}

// decrypt test with 5mb file
pub fn dec_benchmark_large(c: &mut Criterion) {
    c.bench_function("decrypt Shakespeare", |b| {
        b.iter(|| crate::filecrypt::decrypt_file(SHAKESPEARE_CRYPT, None))
    });
}

// test generating a hash
pub fn test_compute_hash(c: &mut Criterion) {
    let contents: Vec<u8> = std::fs::read(DRACULA).unwrap();

    c.bench_function("computing 32-bit hash", |b| {
        b.iter(|| crate::encryption::compute_hash(&contents))
    });
}

// test generation of a 26 digit uuid
pub fn test_generate_uuid(c: &mut Criterion) {
    c.bench_function("generate 26 digit uuid", |b| b.iter(|| generate_uuid()));
}

pub fn test_zip(c: &mut Criterion) {
    let contents = get_file_bytes(DRACULA);
    c.bench_function("zip dracula.txt", |b| {
        b.iter(|| crate::encryption::compress(contents.as_slice(), 3))
    });
}

pub fn test_zip_large(c: &mut Criterion) {
    let contents = get_file_bytes(SHAKESPEARE);
    c.bench_function("zip Shakespeare.txt", |b| {
        b.iter(|| crate::encryption::compress(contents.as_slice(), 3))
    });
}

pub fn cleanup(_c: &mut Criterion) {
    _ = std::fs::remove_file(DRACULA_CRYPT);
    _ = std::fs::remove_file(SHAKESPEARE_CRYPT);
    _ = std::fs::remove_file(DRACULA_DECRYPT);
    _ = std::fs::remove_file(SHAKESPEARE_DECRYPT);
    // _ = std::fs::remove_dir(SAVE_PATH);
}

criterion_group!(
    benches,
    test_zip,
    enc_benchmark,
    dracula_content_encryption,
    shakespeare_content_encryption,
    enc_benchmark_large,
    dec_benchmark,
    dec_benchmark_large,
    test_compute_hash,
    test_generate_uuid,
    cleanup
);
criterion_main!(benches);
