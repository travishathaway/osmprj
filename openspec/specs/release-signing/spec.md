### Requirement: Checksum file is signed with cosign keyless signing
The release pipeline SHALL sign the `checksums.sha256` file using cosign keyless signing backed by GitHub Actions OIDC, producing a `checksums.sha256.bundle` file uploaded as a release asset.

#### Scenario: Signature bundle uploaded to release
- **WHEN** a new release is published
- **THEN** a `checksums.sha256.bundle` file is present as a release asset alongside `checksums.sha256`

#### Scenario: Signature is tied to the release workflow identity
- **WHEN** the signature is generated
- **THEN** it is bound to the certificate identity `https://github.com/travishathaway/osmprj/.github/workflows/release.yml@refs/tags/<tag>` and OIDC issuer `https://token.actions.githubusercontent.com`

### Requirement: Users can verify release provenance with cosign
The signing artifacts SHALL enable users to cryptographically verify that a release was produced by the official GitHub Actions workflow.

#### Scenario: User verifies checksum file provenance
- **WHEN** a user runs `cosign verify-blob` with the bundle, the expected certificate identity, and the OIDC issuer against `checksums.sha256`
- **THEN** the command exits 0 confirming the signature is valid

### Requirement: Signing requires no long-lived secrets
The signing process SHALL use GitHub Actions OIDC for keyless signing and SHALL NOT require any manually managed signing keys or secrets.

#### Scenario: Workflow runs without signing secrets
- **WHEN** the release workflow runs
- **THEN** cosign signing completes successfully using only the `id-token: write` permission with no additional secrets configured
