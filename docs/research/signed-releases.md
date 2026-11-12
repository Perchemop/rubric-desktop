# Research note — signed releases + checksum-verified auto-update

Status: **approved 2026-11-09 (approver initials `ld`)**, recorded in
`docs/research/README.md` (guide §2.3, §8.4). The implementation plan at the
end is an input to a dependent Task **only** because that approval is recorded
(with a date and initials) here and in the index. Implementation remains
queued as its own future slices — nothing has been built yet.

- Scope: how Rubric signs its release artefacts and how the app verifies a
  downloaded update before applying it.
- Requirement: guide §12.1 suite standard — *"signed releases + `SHA256SUMS.txt`,
  in-app verify"* — currently recorded as **Deferred** in
  `docs/guide-compliance.md` and open in `ROADMAP.md` / `IMPLEMENTATION_PLAN.md`.
- Original summary written for Rubric; public URLs only; no vendor text, icons,
  layouts or algorithms reproduced.

---

## 1. Where Rubric is today

- **Release pipeline** (`.github/workflows/release.yml`): on a `v*` tag it builds
  four per-target zips (`linux-x86_64`, `linux-arm64`, `macos-arm64`,
  `windows-x86_64`) and attaches them to a GitHub Release, together with a
  `SHA256SUMS` file computed with `sha256sum` over every archive. A manual
  `workflow_dispatch` run publishes a pre-release. There is **no signing step**
  and no key material in CI.
- **Native installers** (`.github/workflows/package.yml`): Linux AppImage/tarball,
  macOS `.dmg`, Windows `.msi`. These are built on the same tag and are
  explicitly **unsigned** (Q-PACK-1 in `docs/assumptions-and-questions.md`).
- **`crates/update-check`**: queries
  `GET https://api.github.com/repos/Perchemop/rubric-desktop/releases/latest`,
  parses a four-field subset (`tag_name`, `html_url`, `draft`, `prerelease`),
  compares semantic versions, caches for one hour, and **fails soft** to
  `UpdateStatus::Unknown` on any network/parse problem. It **does not download
  anything** and has no concept of artefacts, checksums or signatures.
- **`crates/crash-report`**: unrelated to updates, but it establishes the
  relevant suite principle already — *no privileged secret is ever embedded in
  the client*; a user-supplied token is used for the one privileged action
  (issue creation). The update path must hold that line too: an **Ed25519 public
  key is not a secret** and is safe to embed, while the signing **private key**
  must exist only in CI secrets.
- **In-app UI** (`apps/rubric/src/ui/app.rs`): a "Check for Updates" window and
  the Release Notes popup only ever **open the release page in a browser**
  (`open_url`). There is no download, extract, replace or installer hand-off.
- **Security posture** (`SECURITY.md`): "the distribution server is GitHub
  Releases, signed by GitHub's TLS only … `SHA256SUMS` for transport-integrity
  verification, **not signing**."

So the gap is exactly two things: (a) produce a cryptographically signed
manifest in CI, and (b) teach the app to verify that signature and the checksums
before it offers to apply an update. Everything else (download loop, platform
apply) is design we get to choose.

## 2. Threat model we are closing

Without signing, trust in an artefact rests entirely on GitHub's TLS and on the
single maintainer's GitHub account. Three realistic attacks motivate the work:

1. **Compromised release account / release tampering.** An attacker with a
   stolen session or a compromised maintainer machine can upload a malicious
   archive to the Release. TLS and `sha256sum` do not help — the attacker
   simply recomputes the checksums. A signature made with a key the attacker
   does not hold fails.
2. **A malicious mirror or CDN.** If an artefact is fetched from anywhere other
   than the exact canonical URL, an unsigned download is trivially swappable.
3. **Network/transport tampering on the download path** (captive portals, proxy
   injection). SHA-256 over a *signed* manifest defeats this even if the
   transport is not trustworthy.

Non-goal: this does not defend against a compromised **signing key**. Key
custody (offline backup, rotation — §6/§7) reduces the blast radius but a
stolen private key is game over until rotation, exactly as with any signing
scheme.

## 3. Sources (public)

- **Minisign** — the tool and its documentation:
  <https://jedisct1.github.io/minisign/> · repo
  <https://github.com/jedisct1/minisign> · man page
  <https://github.com/jedisct1/minisign/blob/master/share/man/man1/minisign.1>.
  Ed25519 signatures; `-G` keygen, `-S` sign, `-V` verify; public key is a short
  base64 string, signature file `<file>.minisig`. Compatible with OpenBSD
  `signify`. A `minizign` (Zig) implementation is the upstream-recommended
  successor for new features.
- **`minisign` (Rust crate)** — sign **and** verify, pure Rust, MIT:
  <https://crates.io/crates/minisign> · repo
  <https://github.com/jedisct1/rust-minisign>. Latest `0.10.0` (2026-09-25).
- **`minisign-verify` (Rust crate)** — verify-only, **zero dependencies**, MIT,
  ~2.5k lines, streaming verification:
  <https://crates.io/crates/minisign-verify> · docs
  <https://docs.rs/minisign-verify> · repo
  <https://github.com/jedisct1/rust-minisign-verify>. Latest `0.3.0`
  (2026-09-25). API: `PublicKey::{from_base64,from_file}`,
  `Signature::{decode,from_file}`, `PublicKey::verify(data, &sig, prehashed)`,
  and `verify_stream` for large inputs.
- **cosign / Sigstore** — `cosign sign-blob` / `verify-blob`:
  <https://github.com/sigstore/cosign> · docs
  <https://github.com/sigstore/cosign/blob/main/doc/cosign_verify-blob.md> ·
  Sigstore docs <https://docs.sigstore.dev/>. Two modes: **keyless** (OIDC
  identity + Fulcio short-lived cert + Rekor transparency log; verification
  needs the bundle/trusted-root and, for identity checks, network-reachable
  roots) and **key-based** (a key pair or a KMS URI).
- **`sigstore` (Rust crate)** — <https://crates.io/crates/sigstore>, repo
  <https://github.com/sigstore/sigstore-rs>. Latest `0.15.0` (2026-11-05); the
  crate describes itself as **"experimental"** and its default feature set pulls
  in `reqwest`/`openssl`/`oci-client`/`openidconnect`/Rekor/Fulcio.
- **GitHub Actions artefact attestations / provenance** —
  <https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations>
  (build provenance, `actions/attest-build-provenance`). Relevant as a
  **complement** (proves *where* an artefact was built), not as the app-side
  signature check.

## 4. Evaluation: minisign vs cosign

The decisive question is *what the desktop app must do at verify time*. It must
verify an offline download, on the user's machine, with no server round-trip,
no OCI registry, and no dependency on the maintainer's OIDC identity being
reachable. That is a much smaller problem than container-supply-chain signing,
and it points clearly at a long-lived Ed25519 key and a tiny verifier.

| Criterion | **minisign** | **cosign** |
|---|---|---|
| Signature primitive | Ed25519 (deterministic) | ECDSA P-256 / keyless certs |
| Key model | One long-lived Ed25519 key pair | Key pair **or** keyless OIDC (Fulcio cert, Rekor log) |
| Offline verification in-app | **Yes** — pure Rust, zero-dep `minisign-verify` | Key-based: needs the sigstore stack; keyless: needs bundle + trust root (and, for identity, network) |
| Rust verifier cost | `minisign-verify` (2.5k LOC, no deps) or `minisign` | `sigstore` crate is self-described **experimental**, heavy tree |
| Fits plain GitHub Release zips | **Yes** — sign the `SHA256SUMS` file | Built around OCI images; blob mode works but is a poorer fit |
| CI tooling | one small static CLI, easy to pin + checksum | Go binary + (keyless) GitHub OIDC wiring |
| Transparency / audit trail | none (trust the embedded key) | Rekor transparency log (keyless) |
| Key rotation | add a key to an embedded trusted-key set | swap key / rotate identity |
| Failure mode on user machine | deterministic, offline | keyless verification can fail offline |

**Reading.** cosign's headline advantage — keyless signing with a public
transparency log — is exactly the part that is *hardest* to verify offline in a
desktop app, and its Rust support is explicitly experimental. Rubric distributes
plain zips, not container images; it has a single maintainer and no KMS. A
long-lived Ed25519 key with a ~2.5k-line, zero-dependency, MIT verifier is the
proportionate choice and keeps the app's supply-chain surface small — which is
consistent with the project's existing posture (pure-Rust migration away from
PDFium, `cargo-deny` licence gate, no embedded secrets).

The two are not mutually exclusive: cosign/Sigstore **attestations** could later
be added purely as a CI-side provenance record, without becoming an app-side
dependency. That is recorded here as a future option, not part of this design.

### Considered but rejected

- **Raw SHA-256 only (status quo).** No authenticity — checksums from the same
  channel as the artefact add nothing against attacker #1.
- **GPG/PGP.** Works, but a *worse* developer- and verifier-experience: large
  key material, `gpg` in the loop, and no small zero-dependency Rust verifier.
- **A bespoke Ed25519 implementation with `ed25519-dalek` + a hand-rolled
  signature file format.** Reinvents a well-understood, already-audited format;
  more code to get wrong. minisign is the standard-shaped version of this.
- **`minisign` crate in-app.** Gives signing (unneeded in the client) and pulls
  more code; use it only if a Rust-based CI signing helper is ever wanted.
  `minisign-verify` (verify-only) is the right client dependency.

## 5. Decision

1. **Sign with minisign** (Ed25519). One `SHA256SUMS` manifest is signed; the
   signature covers every artefact at once. Per-artefact signatures are
   unnecessary and would multiply the download surface.
2. **Client verifies with `minisign-verify`** (MIT, zero deps) against a
   **public key embedded at compile time**. The app never holds a private key.
3. **Verification order is strict and fail-closed**: verify signature over the
   manifest → then verify each artefact's SHA-256 against the verified manifest
   → only then expose an "apply/install" affordance. Any failure is an honest
   refusal, never a silent fallback to "just install it anyway".
4. **A release must never publish unsigned.** Signing is a required step in the
   release job; if it fails, the job fails.
5. **cosign/Sigstore stays a possible CI-side provenance complement**, not a
   runtime dependency.

## 6. Design — CI pipeline for signed releases

Changes live in `.github/workflows/release.yml`, in the `publish` job, after the
existing "Collect artifacts" step that builds `dist/SHA256SUMS`.

### 6.1 Artefact layout per release

| File | Purpose |
|---|---|
| `rubric-<platform>.zip` (×4) | the build outputs, unchanged |
| `SHA256SUMS` | existing manifest; **kept** for backward compatibility |
| `SHA256SUMS.txt` | a byte-identical copy of `SHA256SUMS`, published to satisfy the guide's literal `SHA256SUMS.txt` wording |
| `SHA256SUMS.minisig` | the minisign signature **over `SHA256SUMS`** (the canonical manifest) |
| `rubric.pub` | the public key, published for humans verifying by hand (also embedded in the app) |

The verifier accepts either manifest name, so the client is robust to the guide's
naming and to the existing `SHA256SUMS` consumers (`docs/`, `PROGRESS.md`,
`ROADMAP.md`, the installer-verification task).

### 6.2 Steps (conceptual; exact actions pinned by SHA in the workflow)

1. **Install minisign** — fetch a **pinned** minisign release binary for the
   runner and verify a pinned SHA-256 of that binary before using it (a signing
   tool must not be pulled by a mutable tag).
2. **Materialise the key** — write `MINISIGN_SECRET_KEY` from GitHub Actions
   secrets to `$RUNNER_TEMP/rubric.key` with `0600`; never echo it; delete it in
   an `always()` cleanup step.
3. **Sign** — `minisign -Sm dist/SHA256SUMS -s <key> [-t "<comment>"]`, producing
   `dist/SHA256SUMS.minisig`. The passphrase (if the key is encrypted) is read
   from a secret, not a literal. The trusted comment names the repo and tag so a
   human sees provenance on manual verify.
4. **Self-verify in CI (round-trip)** — verify the freshly produced signature
   with the *public* key in the same job, so a broken key or wrong flag fails
   the release instead of shipping a bad signature.
5. **Publish** — upload all of the files from §6.1 via the existing
   `gh release upload … --clobber`.
6. **Make signing unavoidable** — the publish job `needs: build`; the sign step
   is not `continue-on-error`. A missing secret in a fork/dispatch context should
   fail the run on a **tag** but may be skipped for a `workflow_dispatch`
   pre-release (which is not a real release). Record that distinction explicitly
   so a manual smoke run never blocks on secrets it cannot have.

### 6.3 Required secrets / variables

| Name | Kind | Notes |
|---|---|---|
| `MINISIGN_SECRET_KEY` | Actions **secret** | the full `minisign.key` text (encrypted form) |
| `MINISIGN_PASSWORD` | Actions **secret** | passphrase for that key; empty if key is unencrypted (encouraged to set one) |
| `RUBRIC_RELEASE_PUBLIC_KEY` | repo **variable** | the base64 public key, used for the CI self-verify step and printed in release notes |

`docs/qc/checklist.md` ("Secrets: no keys/tokens in git…") gains no new burden:
the private key is a GitHub secret and the public key is not a secret.

## 7. Design — key management and rotation

### 7.1 Custody

- **Private key** (`minisign.key`): passphrase-protected; held **only** as a
  GitHub Actions secret and in a maintainer offline backup (password manager /
  encrypted store). It is never committed, never in a build artifact, never in
  the app.
- **Public key** (`minisign.pub`): committed as `apps/rubric/assets/rubric.pub`
  and `include_str!`-ed into the binary; also published as `rubric.pub` on each
  release and shown in the app's Release Notes popup for out-of-band comparison.
- The public key string also lives in `docs/security/` (or `SECURITY.md`) for
  users verifying a download by hand.

### 7.2 Embedded trusted-key set

Instead of a single hard-coded key, the app embeds an **ordered list**:

```
trusted keys = [ key_1 (active, since vX), ... ]   // old retained for older sigs
```

The verifier accepts a signature whose **key id** matches any entry in the list.
minisign signatures and public keys both carry an 8-byte key id, so a signature
made by an unknown key is rejected even though it is otherwise well-formed.

### 7.3 Rotation procedure (dual-sign / overlap)

Rotation must never produce a gap where releases verify against nothing:

1. **Add the new public key** to the embedded trusted-key set (and to
   `rubric.pub` docs) in a normal app release; ship it *before* the CI switch.
   From here the app trusts both old and new.
2. **Switch CI** to sign with the new private key (`MINISIGN_SECRET_KEY`
   updated). Optionally sign with **both** keys for one release (publish both
   `.minisig` files) so clients that predate step 1 still verify — the
   dual-signature window.
3. **Remove the old key** from the embedded set in a later release, once no
   supported client needs it.
4. **Revoke out-of-band**: if the old key is *suspected compromised*, skip the
   overlap, publish a security advisory, and treat the rotation as an incident
   (`SECURITY.md` "Incident response" — fix, disclose, notify).

Record every key's creation, first-use tag and retirement in a short
`docs/security/signing-keys.md` table so the rotation history is auditable.

## 8. Design — in-app verification flow

### 8.1 Purity boundary (project rule)

Keep the cryptography and parsing **pure and headless**; keep network and file
I/O behind a thin shell. Concretely, two layers:

- **Pure layer** (no I/O, `thiserror` errors, serde types, unit-testable) — new
  modules in `crates/update-check` so the whole suite reuses them (guide §7):
  - `manifest.rs` — parse `SHA256SUMS`/`SHA256SUMS.txt` into
    `Vec<ChecksumEntry { sha256: [u8;32], name: String }>`; reject malformed
    lines, duplicate names, and entries that try to escape the download dir
    (path traversal).
  - `signature.rs` — wrap `minisign_verify`: decode the signature, check its
    key id against the embedded trusted set, verify over the manifest bytes.
  - `verify.rs` — the orchestration as a pure state machine:
    `verify_update(manifest_bytes, sig_bytes, trusted_keys, artifacts: &[(name, sha256)]) -> Result<VerifiedUpdate, VerifyError>`.
  - `keys.rs` — the embedded trusted-key list + the parse of a `minisign.pub`.
  - `sha256.rs` — a thin wrapper over `sha2` (add `sha2` to the crate).
- **Impure shell** (kept out of the pure layer):
  - `crates/update-check` gains an optional download helper (or a new
    `crates/update-apply`) that fetches the manifest, signature and chosen
    artefact with `ureq` (already a dependency), hashes the streamed artefact,
    and calls the pure verifier.
  - The app owns the UX, the temp directory and the platform hand-off.

If the pure logic is judged to belong to the *app's* domain instead of the suite
crate, the same types move to `apps/rubric/src/domain/update.rs` with the same
signatures; the important invariant is that no `std::fs`/`ureq`/time call sits
inside the verifier.

### 8.2 Extended `UpdateStatus`

Add an `Available` payload that carries what verification needs, and a distinct
verified state, while keeping `Unknown` for fail-soft network problems:

- `Available { version, release_json }` — a newer release exists; artefacts not
  yet inspected.
- `Verified { version, artifact_name }` — signature + checksum both passed; the
  app may offer to apply.
- `Untrusted { version, reason }` — a newer release exists but verification
  **failed** (bad/missing signature, unknown key, checksum mismatch). The app
  shows a prominent warning and links to the release page; it must **not** offer
  apply. This is a distinct, deliberate state — never collapsed into `Unknown`.

All new types derive `Serialize, Deserialize` (project rule) and get serde
roundtrip tests, even though they are runtime-only today.

### 8.3 Step-by-step

1. User triggers a check (Help ▸ Check for Updates, or the Release Notes popup).
2. Worker thread fetches the release JSON, including the `assets[]` list with
   `name` and `browser_download_url` (extend the `Release` struct — today it
   ignores assets).
3. Fetch `SHA256SUMS` (or `.txt`) and `SHA256SUMS.minisig` into memory/temp.
4. **Pure verify** the signature over the manifest against the embedded trusted
   set. Fail → `Untrusted`.
5. Parse the manifest; select the entry for the running platform's asset name.
   Missing → `Untrusted` (an unsigned release the client cannot attest).
6. Download the chosen artefact to a temp dir under the OS cache dir; stream it
   through SHA-256. Compare to the manifest entry. Fail → `Untrusted`.
7. Only now show: version, which key signed it ("verified with Rubric release
   key <id8>"), and the artifact that passed. Offer **Apply** (phase-dependent,
   §8.4) and **Open release page**.
8. Delete the temp files on cancel or failure; keep nothing after apply.

### 8.4 Apply strategy (phased; the verifier is a prerequisite for all of it)

Verification is the hard requirement; *applying* is platform-specific and can
land in stages. Nothing below runs before step 6 passes.

- **Phase 1 — verify + hand off.** Download and verify, then open the OS
  installer/package (`.dmg`, `.msi`, AppImage). Honest, low-risk, no
  self-modification. This alone closes the guide standard's "in-app verify"
  clause for the current link-to-release flow.
- **Phase 2 — portable self-replace.** For the portable zip/tarball builds,
  replace the running binary on quit (write-new-then-swap; on Windows schedule
  the swap after exit, since the running exe is locked). Requires the verified
  artifact plus a rollback copy.
- **Phase 3 (optional) — native updater integration.** AppImage update
  information, Sparkle on macOS, or a Windows updater, each still gated on the
  signature check.

## 9. Security properties and non-goals

**Guarantees if implemented as designed**

- An artefact whose manifest signature does not verify against an embedded key
  is never offered for apply, regardless of transport.
- A modified artefact fails the SHA-256 check even if the manifest signature is
  valid.
- A signature from an unknown/retired key is rejected by key id.
- No private key ever ships in the app or a build artifact; the client can only
  *verify*.

**Explicit non-goals**

- Defending against a stolen signing key (mitigated by rotation/incident
  response, not solved).
- Reproducible builds / build-provenance attestation (cosign or GitHub artefact
  attestations could add this later, CI-side).
- Code-signing/notarisation of the OS installers (that is Q-PACK-1, a separate
  trust layer for Gatekeeper/SmartScreen).
- Silent/background updates. Every apply is user-initiated and consent-gated.

## 10. Acceptance criteria

**Slice A — signed releases (CI only)**

- [ ] `release.yml` installs a **pinned, checksum-verified** minisign binary and
      signs `SHA256SUMS`, producing `SHA256SUMS.minisig`.
- [ ] `SHA256SUMS.txt` (byte-identical to `SHA256SUMS`) and `rubric.pub` are
      published on every real release.
- [ ] The job self-verifies the signature with the public key and **fails** if
      verification fails; a missing secret fails a tag run.
- [ ] A manually dispatched pre-release is documented as unsigned-or-skipped,
      and cannot masquerade as a real signed release.
- [ ] A human can verify a downloaded release with
      `minisign -Vm SHA256SUMS -p rubric.pub` and with `sha256sum -c`.

**Slice B — pure verifier (domain, headless)**

- [ ] `minisign-verify` + `sha2` added; licenses MIT / MIT OR Apache-2.0, both
      already allowed by `deny.toml`; recorded in `ATTRIBUTION.md`.
- [ ] Manifest parsing rejects malformed lines, duplicate names and path
      traversal; round-trips through serde where types are serialisable.
- [ ] Signature verification respects the trusted-key list and **rejects an
      unknown key id**.
- [ ] `verify_update` returns `Verified` only when the signature verifies
      **and** every supplied artefact hash matches; otherwise a typed error.
- [ ] Unit tests cover: valid fixture, flipped-byte artefact, wrong key, tampered
      manifest, missing asset, empty/garbage input, and the prehashed-vs-legacy
      flag. A **real signature fixture** (committed, generated by the pinned
      minisign CLI with a throwaway test key) pins the flag semantics.

**Slice C — app UX**

- [ ] `Release` parsing reads `assets[]`; the check fetches manifest + signature.
- [ ] A successful check that fails verification shows `Untrusted` with a plain
      explanation and **no apply button**.
- [ ] A verified check shows the signing key id and the verify-before-apply
      affordance; temp files are cleaned on cancel/failure.
- [ ] `SECURITY.md` and `docs/guide-compliance.md` (the §12.1 row) are updated
      from **Deferred** to **Met** with the CI run id and a manual
      `minisign -V` transcript.
- [ ] `ROADMAP.md` / `IMPLEMENTATION_PLAN.md` / `PROGRESS.md` / `CHANGELOG.md`
      updated from evidence only.

## 11. Implementation plan (vertical slices)

Each slice is one Task → PR → CI-green → merge, per the project operating model.
File ownership is disjoint per slice.

| # | Slice | Files | Notes |
|---|---|---|---|
| A | Sign releases in CI | `.github/workflows/release.yml`, `docs/security/signing-keys.md`, `docs/qc/checklist.md` | needs the two secrets created by the maintainer (human action) |
| B1 | Pure manifest + sha256 types | `crates/update-check/src/manifest.rs`, `sha256.rs`, `keys.rs` | `sha2` dep; unit + serde tests |
| B2 | Pure signature verify | `crates/update-check/src/signature.rs`, `verify.rs` | `minisign-verify` dep; fixture-based tests |
| C1 | Fetch + verify wiring | `crates/update-check/src/lib.rs` (assets in `Release`), download helper | ureq already present |
| C2 | App UX (`Untrusted`, verified state, cleanup) | `apps/rubric/src/ui/app.rs`, `app_menu.rs` | no apply before verify |
| D | Apply/hand-off (**phase 1**) | `apps/rubric/src/` (new `update_apply` module) | open verified installer; self-replace is a later slice |
| E | Docs + compliance closure | `SECURITY.md`, `docs/guide-compliance.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `PROGRESS.md`, `CHANGELOG.md`, `ATTRIBUTION.md` | evidence-linked |

**Human action required before Slice A can run:** generate the key pair, store
the private key + passphrase as Actions secrets, commit the public key, and
record the key id + date in `docs/security/signing-keys.md`. This is the one
irreversible, owner-only step.

## 12. Risks / details to confirm during implementation

- **Prehashed vs legacy signature flag.** Modern minisign signs *pre-hashed* by
  default; `minisign-verify::PublicKey::verify(data, sig, prehashed)` takes the
  flag explicitly, and streaming verification only supports pre-hashed. Pin the
  CI minisign version, and pin the semantics with a committed fixture so the
  flag can never silently drift.
- **`SHA256SUMS` naming.** The guide says `SHA256SUMS.txt`; the repo publishes
  `SHA256SUMS`. Publish both (identical bytes) and have the verifier accept
  either — no rename churn for existing consumers.
- **`Release.assets` shape.** GitHub returns assets with `name` and
  `browser_download_url`; the current struct ignores them. Confirm exact field
  names against the live payload (there is a deserialisation test pattern in
  `crates/update-check/src/lib.rs` to extend).
- **Windows self-replace** is the fiddliest part of Phase 2 (locked executable);
  it is deliberately out of the first apply slice.
- **Secret hygiene in CI** — the key file must be `0600`, never echoed, and
  removed in an `always()` step; `scripts/scan-release-assets.sh` and
  `scripts/scan-personal-data.sh` continue to run over `dist/` before publish.
- **Fork PRs** cannot access repo secrets; the sign step must skip cleanly on
  non-release contexts while remaining mandatory on tags.

## 13. Clean-room & legal notes

- All code written for Rubric will be original; the vaunted crates
  (`minisign-verify` MIT, `minisign` MIT, `sha2` MIT/Apache-2.0) are permissive
  and already within the `deny.toml` allow-list. Add them to `ATTRIBUTION.md`.
- The minisign **CLI** fetched in CI is a build tool, not a Rust dependency;
  record its licence at the pinned tag and its checksum in the workflow, and
  only add it to `ATTRIBUTION.md` if it is ever bundled into an artefact.
- Proposed public key string lives in the repo by design; it is **not** a
  secret, but do not paste any private-key material, passphrase, or token into
  docs, fixtures or CI logs (guide II.29). Test fixtures must use a throwaway
  key generated solely for tests.

## 14. Approval

Approval is recorded (guide §2.3, §8.4); the checklist below tracks the
decisions still open before Slice A/B/C implementation Tasks may depend on it.

- [x] Approved — date: 2026-11-09  approver initials: ld
- [ ] Decision on the tool: **minisign** (recommended) · cosign · other: ______
- [ ] Commitment to create the Actions secrets + offline key backup (owner-only)

The index row in `docs/research/README.md` and the dated approval above are
recorded; proceed with Slice A once the tool decision and the key-backup
commitment are made.
