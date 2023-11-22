use chacha20poly1305::{ChaCha20Poly1305, aead::{OsRng, generic_array::sequence::GenericSequence}, KeyInit, Key};
use rand::RngCore;
// use rand_core::{RngCore, OsRng};


fn main() {

    let mut k = [0u8; 32]; 
    OsRng.fill_bytes(&mut k);
    println!("random k: {:?}", k);

    for _ in 0..5 {
        let key = Key::from_slice(&k);
        println!("key: {:?}", key);
    }
}

// pub fn generate(&mut self) {

//     let mut k = [0u8; 16];
//     OsRng.fill_bytes(&mut key);


//     let k_rand = OsRng;
//     let n_rand = OsRng;
//     let key = ChaCha20Poly1305::generate_key(&mut OsRng).as_slice();
//     let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng).as_slice(); // 192-bits; unique per message  
// }