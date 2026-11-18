# Release signing state

Recorded 2026-11-18 (guide §7.2).

## Public key

`assets/release-signing.pub` is the pinned minisign (Ed25519) public key. Its
contents, verbatim:

```
untrusted comment: minisign public key 5DDF86377FAD5DB1
RWSxXa1/N4bfXT8EIrnSfxmAaIx9wSn25VD5lCPVF7ZXWNDMt14Bw6Rr
```

Key ID from the comment: `5DDF86377FAD5DB1`. The public key is bundled with the
app, so a key rotation requires a new app release carrying the new key.

## What ships on each release

The release workflow (`.github/workflows/release.yml`) generates `SHA256SUMS`
over every archive, publishes `SHA256SUMS.txt` as a byte-identical copy, signs
that file with minisign, and attaches `SHA256SUMS.txt.minisig` beside it. The
job fails if the signing secrets or the signature file are missing, so a
release without a signature does not ship. Signing first appears in the
changelog under `[0.4.0]` (PR #175); `CHANGELOG.md` has no `v0.4.9` entry, so no
earlier "since v0.4.9" boundary is claimed here.

## Verification in the app

`crates/update-check` owns verification:

- `download_and_verify_asset` (in `src/lib.rs`) downloads a release asset to a
  temp file, verifies its SHA-256 against the release's `SHA256SUMS.txt`
  checksum manifest, and returns the local path. Network problems, a missing
  manifest, a missing entry for the asset, and a checksum mismatch are all
  returned as `Err(String)` for the caller to surface — never a silent failure.
- The manifest's minisign signature is checked in the same crate by
  `fetch_and_verify_signature` / `signature::verify_manifest`, with
  `SignatureState::{Verified, Failed, Missing}`. A missing or failed signature
  gates the install path.

## Private key and rotation

The password-encrypted private key is held by the owner outside the repository:
the release workflow reads it from the Actions secrets `MINISIGN_SECRET_KEY`
(the whole key file, including its untrusted-comment line) and
`MINISIGN_PASSWORD`. It is never committed to git or shipped in a binary.

If compromise is suspected, follow guide §7.2:

1. Publish a new app release bundling a freshly generated public key **before**
   any further artefact is signed with the old key.
2. Mark the affected release(s) with a clear "Security" banner in the release
   notes and the in-app release-notes popup.
3. Update this file with the old fingerprint, the new fingerprint, and the
   reason for the rotation.
4. Rotate the secret in the release workflow (`MINISIGN_SECRET_KEY` and
   `MINISIGN_PASSWORD`).

Never silently re-sign existing artefacts with a new key — that hides the
incident and creates ambiguity.
