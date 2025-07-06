// ANCHOR: demo_test
use aead::{KeyInit, OsRng};
use nonce_typing::{EncryptionNonce, NonceSafeAead};

const PLAINTEXT_MSG: &[u8; 86] = b"Two cryptographers walk into a bar. \
    Nobody else has a clue what they're talking about.";

#[test]
fn nonce_safe_xchacha20poly1305() {
    use chacha20poly1305::XChaCha20Poly1305;

    let key = XChaCha20Poly1305::generate_key(&mut OsRng);
    let cipher = XChaCha20Poly1305::new(&key);
    let enc_nonce = EncryptionNonce::<XChaCha20Poly1305>::generate_nonce(&mut OsRng);

    let (ciphertext, dec_nonce) = cipher
        .nonce_safe_encrypt(enc_nonce, PLAINTEXT_MSG.as_ref())
        .unwrap();

    let plaintext = cipher.decrypt(&dec_nonce, ciphertext.as_ref()).unwrap();

    assert_eq!(&plaintext, PLAINTEXT_MSG);
}
// ANCHOR_END: demo_test

#[test]
fn nonce_safe_aes256gcm() {
    use aes_gcm::Aes256Gcm;

    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let enc_nonce = EncryptionNonce::<Aes256Gcm>::generate_nonce(&mut OsRng);

    let (ciphertext, dec_nonce) = cipher
        .nonce_safe_encrypt(enc_nonce, PLAINTEXT_MSG.as_ref())
        .unwrap();

    let plaintext = cipher.decrypt(&dec_nonce, ciphertext.as_ref()).unwrap();

    assert_eq!(&plaintext, PLAINTEXT_MSG);
}

// Note: in the SIV case, nonce-reuse only leaks message equivalences - doesn't allow plaintext or key recovery.
#[test]
fn nonce_safe_aes256siv() {
    use aes_siv::Aes256SivAead;

    let key = Aes256SivAead::generate_key(&mut OsRng);
    let cipher = Aes256SivAead::new(&key);
    let enc_nonce = EncryptionNonce::<Aes256SivAead>::generate_nonce(&mut OsRng);

    let (ciphertext, dec_nonce) = cipher
        .nonce_safe_encrypt(enc_nonce, PLAINTEXT_MSG.as_ref())
        .unwrap();

    let plaintext = cipher.decrypt(&dec_nonce, ciphertext.as_ref()).unwrap();

    assert_eq!(&plaintext, PLAINTEXT_MSG);
}
