//! Minisign signature verification for the release checksum manifest.
//!
//! Pure module: no I/O and no network. The caller hands over the manifest
//! text and the `.minisig` signature file text; this module decides whether
//! that manifest is signed by the trusted Rubric release key.
//!
//! Fail-closed mapping: only a signature that verifies against
//! [`RELEASE_SIGNING_PUBLIC_KEY`] yields [`SignatureState::Verified`]. A
//! tampered manifest, a foreign key, or a malformed signature file all yield
//! [`SignatureState::Failed`] - never a pass, never a panic.

use minisign_verify::{PublicKey, Signature};

/// The Rubric release public key: the base64 line of
/// `assets/release-signing.pub` (key id [`RELEASE_SIGNING_KEY_ID`]).
///
/// A public key is not a secret; the signing private key lives only in the
/// release CI secrets and the maintainer's offline backup.
pub const RELEASE_SIGNING_PUBLIC_KEY: &str =
    "RWSxXa1/N4bfXT8EIrnSfxmAaIx9wSn25VD5lCPVF7ZXWNDMt14Bw6Rr";

/// The key id of [`RELEASE_SIGNING_PUBLIC_KEY`]. Reported on success because
/// this is the only key the app trusts.
pub const RELEASE_SIGNING_KEY_ID: &str = "5DDF86377FAD5DB1";

/// Outcome of checking the release manifest's minisign signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureState {
    /// The manifest is signed by the trusted release key.
    Verified {
        /// The key id that verified (always [`RELEASE_SIGNING_KEY_ID`]).
        key_id: String,
    },
    /// A signature was present but did not verify: tampered manifest, wrong
    /// key, or malformed signature file. Fail closed.
    Failed,
    /// No signature was available (fetch failed or the release has none).
    /// Never a pass: an unverified release is simply not attested.
    Missing,
}

/// Verify `manifest` against the embedded Rubric release key.
pub fn verify_manifest(manifest: &str, signature_file: &str) -> SignatureState {
    verify_manifest_with_key(manifest, signature_file, RELEASE_SIGNING_PUBLIC_KEY)
}

/// Core verification with an explicit base64 public key, so tests can pin the
/// signature file format with a dedicated fixture key instead of the trusted
/// release key. Any decode or verification failure is
/// [`SignatureState::Failed`].
pub fn verify_manifest_with_key(
    manifest: &str,
    signature_file: &str,
    key_b64: &str,
) -> SignatureState {
    let Ok(public_key) = PublicKey::from_base64(key_b64) else {
        return SignatureState::Failed;
    };
    let Ok(sig) = Signature::decode(signature_file) else {
        return SignatureState::Failed;
    };
    // `false` accepts only modern (pre-hashed) minisign signatures, which is
    // what the pinned release CI minisign produces; legacy signatures fail
    // closed. The fixture tests below pin this flag with a real signature.
    match public_key.verify(manifest.as_bytes(), &sig, false) {
        Ok(()) => SignatureState::Verified {
            key_id: RELEASE_SIGNING_KEY_ID.to_string(),
        },
        Err(_) => SignatureState::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dedicated throwaway test key (key id CE8651864E544B13), generated with
    /// the minisign CLI solely for these fixtures; the app never trusts it.
    const TEST_PUBLIC_KEY: &str = "RWQTS1ROhlGGzmRJlDODapvZUW147xk0UFYYetb1tg8d1e3X7RQW+u7B";

    /// Two-entry `SHA256SUMS`-shaped manifest signed by `TEST_PUBLIC_KEY`.
    const TEST_MANIFEST: &str = concat!(
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "  rubric-linux-arm64.zip\n",
        "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
        "  rubric-windows-x64.zip\n"
    );

    /// The minisign signature file over `TEST_MANIFEST`, byte-for-byte as the
    /// minisign CLI wrote it: untrusted comment, signature, trusted comment,
    /// global signature.
    const TEST_SIGNATURE_FILE: &str = concat!(
        "untrusted comment: signature from minisign secret key\n",
        "RUQTS1ROhlGGzgpTYfpTkr81Mp3uS86DC3cssp8wxh3zGY7Sf8QYn3dGZo+j0kUZ",
        "XGQQwY9pFIefFNLPejZQIlMTDH5ubqvcUAw=\n",
        "trusted comment: timestamp:1794605453\tfile:test_SHA256SUMS.txt\thashed\n",
        "1UM5CoTnPm2dNcBlyX7rjTe7CYhn9ENPXQklMc0rQJIkxJzystT456BBjCK+gZFvCC",
        "mqZLDR7U26tivyTvamAw=="
    );

    #[test]
    fn verify_manifest_accepts_a_valid_signature() {
        assert_eq!(
            verify_manifest_with_key(TEST_MANIFEST, TEST_SIGNATURE_FILE, TEST_PUBLIC_KEY),
            SignatureState::Verified {
                key_id: RELEASE_SIGNING_KEY_ID.to_string()
            }
        );
    }

    #[test]
    fn verify_manifest_rejects_a_tampered_manifest() {
        // Flip one byte of the first checksum; the signature no longer fits.
        let tampered = TEST_MANIFEST.replacen('e', 'f', 1);
        assert_ne!(tampered, TEST_MANIFEST);
        assert_eq!(
            verify_manifest_with_key(&tampered, TEST_SIGNATURE_FILE, TEST_PUBLIC_KEY),
            SignatureState::Failed
        );
    }

    #[test]
    fn verify_manifest_rejects_a_foreign_key() {
        // The same signature checked against the release key instead of the
        // key that made it: rejected, never passed through.
        assert_eq!(
            verify_manifest_with_key(
                TEST_MANIFEST,
                TEST_SIGNATURE_FILE,
                RELEASE_SIGNING_PUBLIC_KEY
            ),
            SignatureState::Failed
        );
    }

    #[test]
    fn verify_malformed_signature_file_is_failed() {
        for garbage in ["", "not a signature", "untrusted comment: x\n"] {
            assert_eq!(
                verify_manifest_with_key(TEST_MANIFEST, garbage, TEST_PUBLIC_KEY),
                SignatureState::Failed
            );
        }
    }
}
