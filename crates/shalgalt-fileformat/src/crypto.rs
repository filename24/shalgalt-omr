//! Thin `pub(crate)` wrapper over `age` 0.11 — the **only** module that touches the age API.
//!
//! Streams both ways (Rule 1): no `Vec<u8>` buffering of payloads. ASCII armor is always on
//! (master plan §6.3, portability). Keeping every age call here means the rest of the crate
//! stays crypto-library-agnostic and a future algorithm swap touches one file.

use std::io::{Read, Write};

use age::armor::{ArmoredReader, ArmoredWriter, Format};
use age::secrecy::SecretString;

use crate::error::FileFormatError;

/// Wrap a `&str` passphrase in age's re-exported [`SecretString`] (zeroizing). Using age's
/// re-export — not a separate `secrecy` dep — keeps the version in lockstep.
fn secret(passphrase: &str) -> SecretString {
    SecretString::from(passphrase.to_owned())
}

/// The concrete writer type returned by [`encrypting_writer`]. Spelled out so callers can
/// name it: plaintext written here is chunk-encrypted, then ASCII-armored, then forwarded to
/// the inner sink `W`.
pub(crate) type EncryptingWriter<W> = age::stream::StreamWriter<ArmoredWriter<W>>;

/// Wrap `sink` so plaintext written to the returned writer is passphrase-encrypted **and**
/// ASCII-armored. The caller streams plaintext in (e.g. via `io::copy`) and then MUST call
/// [`finish_encrypt`] to flush the final chunk and the armor footer.
pub(crate) fn encrypting_writer<W: Write>(
    sink: W,
    passphrase: &str,
) -> Result<EncryptingWriter<W>, FileFormatError> {
    // Inside-out: armor wraps the sink first, then the age encryptor wraps the armor writer.
    let armored = ArmoredWriter::wrap_output(sink, Format::AsciiArmor)
        .map_err(|e| FileFormatError::Crypto(e.to_string()))?;
    let encryptor = age::Encryptor::with_user_passphrase(secret(passphrase));
    encryptor
        .wrap_output(armored)
        .map_err(|e| FileFormatError::Crypto(e.to_string()))
}

/// Close an [`EncryptingWriter`] in inside-out order: finish the age stream (flush the final
/// ciphertext chunk), then finish the armor writer (emit `-----END AGE ENCRYPTED FILE-----`).
/// Skipping the second finish truncates the armor and produces an unreadable entry. Returns
/// the inner sink so the caller can continue writing the surrounding archive.
pub(crate) fn finish_encrypt<W: Write>(writer: EncryptingWriter<W>) -> Result<W, FileFormatError> {
    let armored = writer
        .finish()
        .map_err(|e| FileFormatError::Crypto(e.to_string()))?;
    armored
        .finish()
        .map_err(|e| FileFormatError::Crypto(e.to_string()))
}

/// Wrap `src` so reads yield decrypted plaintext. `src` may be ASCII-armored or raw binary —
/// [`ArmoredReader`] auto-detects. A **wrong passphrase** surfaces here as
/// [`age::DecryptError::DecryptionFailed`] (scrypt's single-key failure mode in age 0.11),
/// mapped to [`FileFormatError::BadPassphrase`].
pub(crate) fn decrypting_reader<R: Read>(
    src: R,
    passphrase: &str,
) -> Result<impl Read, FileFormatError> {
    let decryptor = age::Decryptor::new(ArmoredReader::new(src)).map_err(map_decrypt_err)?;
    // Only passphrase (scrypt) files are valid here; a non-scrypt age file in a `.shalgalt`
    // means the archive was produced by something else and is malformed for our purposes.
    if !decryptor.is_scrypt() {
        return Err(FileFormatError::Malformed(
            "encrypted entry is not a passphrase (scrypt) age file".to_owned(),
        ));
    }
    let identity = age::scrypt::Identity::new(secret(passphrase));
    decryptor
        .decrypt(std::iter::once(&identity as &dyn age::Identity))
        .map_err(map_decrypt_err)
}

/// Map age decrypt errors to the crate error.
///
/// For a **scrypt (passphrase)** age file the only key material is the passphrase, so every
/// key-unwrap / decryption failure (`DecryptionFailed`, `KeyDecryptionFailed`,
/// `NoMatchingKeys`) means the passphrase was wrong. age 0.11 returns `DecryptionFailed` in
/// practice (verified against the crate); the others are mapped too for robustness across
/// patch releases. Structural failures (`InvalidHeader`, `InvalidMac`, `UnknownFormat`) are
/// genuine corruption, not a passphrase problem, so they stay `Crypto`.
fn map_decrypt_err(e: age::DecryptError) -> FileFormatError {
    match e {
        age::DecryptError::DecryptionFailed
        | age::DecryptError::KeyDecryptionFailed
        | age::DecryptError::NoMatchingKeys => FileFormatError::BadPassphrase,
        // scrypt work factor above the device target — a DoS guard, not a wrong passphrase.
        age::DecryptError::ExcessiveWork { .. } => {
            FileFormatError::Crypto("scrypt work factor exceeds device target".to_owned())
        }
        age::DecryptError::Io(io) => FileFormatError::Io(io),
        // Catch-all keeps us compiling across age patch releases that add variants; the
        // Display message is sanitized and never carries passphrase material.
        other => FileFormatError::Crypto(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Round-trip a payload through encrypt -> armor -> decrypt with the right passphrase.
    #[test]
    fn encrypt_then_decrypt_round_trips() {
        let plaintext = b"the quick brown fox jumps over 13 lazy dogs".to_vec();

        let mut ciphertext = Vec::new();
        {
            let mut enc = encrypting_writer(&mut ciphertext, "correct horse").unwrap();
            enc.write_all(&plaintext).unwrap();
            finish_encrypt(enc).unwrap();
        }
        // Armored output is ASCII text wrapped in the age armor envelope.
        assert!(ciphertext.starts_with(b"-----BEGIN AGE ENCRYPTED FILE-----"));

        let mut decrypted = Vec::new();
        let mut reader = decrypting_reader(Cursor::new(ciphertext), "correct horse").unwrap();
        reader.read_to_end(&mut decrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    /// A wrong passphrase fails at `decrypting_reader` time with the typed error + stable code.
    #[test]
    fn wrong_passphrase_is_bad_passphrase() {
        let mut ciphertext = Vec::new();
        {
            let mut enc = encrypting_writer(&mut ciphertext, "correct").unwrap();
            enc.write_all(b"secret").unwrap();
            finish_encrypt(enc).unwrap();
        }

        // `impl Read` is not `Debug`, so map the Ok side away before asserting on the error.
        let err = decrypting_reader(Cursor::new(ciphertext), "wrong")
            .map(|_| ())
            .unwrap_err();
        assert_eq!(err.code(), "fileformat.bad_passphrase");
        assert!(matches!(err, FileFormatError::BadPassphrase));
    }
}
