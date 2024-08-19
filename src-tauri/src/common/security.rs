//! The purpose of this module is to expose security features needed for the
//! `SystemCommandEffect`. Because users are expected to exchange and import
//! others' Triggers files, a malicious adversary could exploit this and share
//! a Triggers file that uses `SystemCommandEffect` to pwn the user.
//!
//! This risk is mitigated by implementing a secure means of detecting that a
//! particular Triggers file is being loaded from a different machine than the
//! current one. If so, the user must be prompted to manually verify all of the
//! commands being imported (or simply discard all `SystemCommandEffect`s).
//!
//! This algorithm works by detecting a unique machine ID for the system, with
//! help from the `machine-uid` crate. If this is not available for some reason,
//! then the `SystemCommandEffect` feature will be disabled completely. With the
//! unique machine ID, a salted SHA512 checksum is calculated based on its
//! value, and this checksum is used as the seed for a Ed25519 private/public
//! key-pair. When a new `CommandTemplate` is created (for use in a
//! `SystemCommandEffect`), it is given a cryptographic signature based on the
//! private-key resident only in-memory within LogQuest. This signature is
//! serialized along with the `SystemCommandEffect` to disk and checked every
//! time the data is loaded. If the signature verification fails, then the user
//! must intervene to manually approve the `SystemCommandEffect`'s command; this
//! approval allows LogQuest to re-sign the data with the user's own machine-
//! specific private key. This new signature is then saved with the `CommandTemplate`
//! in their Triggers file. Sending the Triggers file to anyone else, or moving
//! the file to another computer, will force the new machine's user to approve
//! the commands for that machine's Triggers file to have new, valid signatures.
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use lazy_static::lazy_static;
use ring::{
  digest::{digest, SHA512},
  signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519},
};
use tracing::error;

lazy_static! {
  static ref MACHINE_ID: Option<[u8; 512 / 8]> = match machine_uid::get() {
    Ok(uid) => {
      let mut uid: Vec<u8> = uid.into_bytes();
      uid.extend_from_slice(b"LogQuest"); // salt
      let checksum = digest(&SHA512, &uid);
      Some(checksum.as_ref().try_into().unwrap()) // unwrap safe here
    }
    Err(e) => {
      error!("Could not determine machine-uid [ ERROR: {e:?} ]");
      None
    }
  };
  static ref MACHINE_KEY_PAIR: Option<Ed25519KeyPair> =
    MACHINE_ID.and_then(|id| Ed25519KeyPair::from_seed_unchecked(&id).ok());
}

pub fn is_crypto_available() -> bool {
  MACHINE_KEY_PAIR.is_some()
}

pub fn sign(data: &str) -> String {
  let signature = MACHINE_KEY_PAIR
    .as_ref()
    .expect("sign() called without checking is_crypto_available()")
    .sign(data.as_bytes());

  URL_SAFE.encode(signature).to_owned()
}

pub fn verify(data: &str, signature: &str) -> bool {
  let Ok(signature) = URL_SAFE.decode(signature) else {
    error!("Failed to decode a signature from Base64! {signature:?}");
    return false;
  };

  let verification_key = MACHINE_KEY_PAIR
    .as_ref()
    .expect("verify() called without checking is_crypto_available()")
    .public_key();

  UnparsedPublicKey::new(&ED25519, verification_key)
    .verify(data.as_bytes(), &signature)
    .is_ok()
}

mod tests {
  #[test]
  fn test_sign_and_verify() {
    assert!(super::is_crypto_available());
    let some_data = "This is a test";
    let sig = super::sign(some_data);
    assert!(super::verify(some_data, &sig));
  }
}
