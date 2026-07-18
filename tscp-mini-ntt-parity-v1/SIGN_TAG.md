# Signing the tscp-mini-ntt-parity-v1 tag

The annotated tag `tscp-mini-ntt-parity-v1` exists on GitHub but is not yet GPG-signed.
A GPG-signed tag binds the SHA-256 content chain to an authorized key holder —
this is what upgrades "auditable reproducibility" to "cryptographic attestation."

## Prerequisites

- GPG key `84692E6294128CC1C4ACCD15E747C3AF22573539` present in keyring
- Key registered on both GitHub accounts (Settings → SSH and GPG keys)
- SSH access to `git@github.com:Triune-Oracle/tscp-anchor.git` working
- `commit.gpgsign = true` set in global git config

## Commands (run in ~/tscp-anchor on Termux or local machine)

```bash
# Pull the latest branch
cd ~/tscp-anchor
git fetch origin
git checkout feat/tscp-mini-ntt-parity-v1
git pull origin feat/tscp-mini-ntt-parity-v1

# Delete the unsigned tag locally (if it exists) and remotely
git tag -d tscp-mini-ntt-parity-v1 2>/dev/null || true
git push origin :refs/tags/tscp-mini-ntt-parity-v1

# Create a GPG-signed annotated tag
git tag -s tscp-mini-ntt-parity-v1 27de2f100b7e0ad2dc4a5ad09899c93ff361f85f \
  -m "tscp-mini-ntt-parity-v1: first NTT evidence object

Provenance chain:
  artifact:        tscp-mini-ntt-parity-v1
  status:          VERIFICATION_PACKAGE_PASS
  checks_passed:   24/24
  sha256_manifest: 2c9125388a2b08c70596f03f06af39e5435841c901e919f6820fbd36a9c5f0d3
  acceptance-evidence: a6006733805a48199de2db0ef5e05f1b07e46c2ff7a5c5d3072607be39d5badf

Chain: git commit -> CI -> vector_generator.py -> verify.py -> 24/24 PASS -> SHA256SUMS -> acceptance-evidence.json
Signed by: 84692E6294128CC1C4ACCD15E747C3AF22573539"

# Verify the signature locally
git tag -v tscp-mini-ntt-parity-v1

# Push the signed tag
git push origin tscp-mini-ntt-parity-v1
```

## Expected output from `git tag -v`

```
object 27de2f100b7e0ad2dc4a5ad09899c93ff361f85f
type commit
tag tscp-mini-ntt-parity-v1
tagger Triune-Oracle <schlagetorren@gmail.com> ...

gpg: Signature made ...
gpg:                using ECDSA key 84692E6294128CC1C4ACCD15E747C3AF22573539
gpg: Good signature from "SEAN CHRISTOPHER SOUTHWICK (https://toolintell.com) <schlagetorren@gmail.com>"
```

## What this achieves

| Layer | Before signing | After signing |
|---|---|---|
| Content integrity | SHA-256 chain | SHA-256 chain |
| Reproducibility | Verified (clean checkout passes) | Verified |
| Identity binding | GitHub account only | GPG key `84692E...` |
| Trust anchor | `git commit` SHA | Signed tag + commit SHA |
| Attestation type | Auditable reproducibility | Cryptographic attestation |

## GitHub verification

Once the signed tag is pushed and the public key is registered on GitHub,
the tag will show a "Verified" badge at:
https://github.com/Triune-Oracle/tscp-anchor/releases/tag/tscp-mini-ntt-parity-v1

The GPG public key to register:
Key ID: `84692E6294128CC1C4ACCD15E747C3AF22573539`
Export: `gpg --armor --export 84692E6294128CC1C4ACCD15E747C3AF22573539`
