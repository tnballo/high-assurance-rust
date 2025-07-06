#![deny(missing_docs)]

//! An alternative to trait [`aead::Aead`] that prevents nonce reuse vulnerabilities.

// ANCHOR: nonce_typing
use aead::{
    Aead, AeadCore, Nonce, Payload,
    rand_core::{CryptoRng, RngCore},
};
use core::error::Error;

/// Can be used in arbitrarily many decryption operations.
/// Its counterpart, [`EncryptionNonce`], can only be used for one encryption operation.
pub type DecryptionNonce<A> = Nonce<A>;

/// A safer nonce type for AEAD. See trait [`NonceSafeAead`].
//
// SECURITY: Intentionally opaque and unique. Do not derive/implement any of:
// `Default`, `Copy`, `Clone`, `Ord`, `Eq`, `Debug`, etc.
pub struct EncryptionNonce<A: AeadCore>(Nonce<A>);

impl<A: AeadCore> EncryptionNonce<A> {
    /// Generate a new random nonce for AEAD-specific encryption.
    pub fn generate_nonce(rng: impl CryptoRng + RngCore) -> Self {
        EncryptionNonce(<A as AeadCore>::generate_nonce(rng))
    }

    /// Crate-private conversion into [`aead::Nonce`].
    //
    // SECURITY: Do not make `pub`, risks reuse with `aead::Aead` APIs.
    fn less_safe_to_raw_nonce(self) -> Nonce<A> {
        self.0
    }
}

/// Nonce-safe AEAD. Guarantees the following properties:
///
/// 1. Nonce is random.
///     * Opaque type with rand-only constructor.
/// 2. Nonce is used in exactly one encryption operation.
///     * Pass-by-value consumption.
///
/// See also: [`EncryptionNonce`] and [`DecryptionNonce`].
pub trait NonceSafeAead {
    /// Encrypt plaintext payload with a random, single-use nonce.
    /// Returns ciphertext bytes and decryption-only nonce.
    fn nonce_safe_encrypt<'msg, 'aad>(
        &self,
        enc_nonce: EncryptionNonce<Self>,
        plaintext: impl Into<Payload<'msg, 'aad>>,
    ) -> Result<(Vec<u8>, DecryptionNonce<Self>), impl Error>
    where
        Self: AeadCore + Aead + Sized,
    {
        let nonce = enc_nonce.less_safe_to_raw_nonce();
        self.encrypt(&nonce, plaintext)
            .map(|ciphertext| (ciphertext, nonce))
    }

    /// Decrypt ciphertext.
    /// Identical to [`aead::Aead::decrypt`], defined so that [`aead::Aead`]
    /// doesn't have to be brought in-scope when using [`NonceSafeAead`].
    //
    // SECURITY: ban import of less safe `aead::Aead` trait.
    fn decrypt<'msg, 'aad>(
        &self,
        dec_nonce: &DecryptionNonce<Self>,
        ciphertext: impl Into<Payload<'msg, 'aad>>,
    ) -> Result<Vec<u8>, impl Error>
    where
        Self: AeadCore + Aead + Sized,
    {
        <Self as Aead>::decrypt(self, dec_nonce, ciphertext)
    }
}

// Use above default impl for below algorithms
impl NonceSafeAead for chacha20poly1305::XChaCha20Poly1305 {}
impl NonceSafeAead for aes_gcm::Aes256Gcm {}
impl NonceSafeAead for aes_siv::Aes256SivAead {}
// ANCHOR_END: nonce_typing
