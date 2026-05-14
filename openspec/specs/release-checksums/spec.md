### Requirement: SHA256 checksums published with every release
The release pipeline SHALL generate a `checksums.sha256` file containing SHA256 hashes for all release installer assets and upload it as a release asset.

#### Scenario: Checksum file uploaded to release
- **WHEN** a new release is published
- **THEN** a `checksums.sha256` file is present as a release asset containing one line per installer file in the format `<hash>  <filename>`

#### Scenario: Checksum covers all platform installers
- **WHEN** the checksum file is generated
- **THEN** it includes entries for `osmprj-linux-64-installer.sh`, `osmprj-linux-aarch64-installer.sh`, `osmprj-osx-arm64-installer.sh`, and `install.sh`

### Requirement: Users can verify installer integrity with checksums
The checksum file SHALL be usable by end users to verify installer integrity using standard tools.

#### Scenario: User verifies a downloaded installer
- **WHEN** a user downloads the installer and `checksums.sha256` from the release page and runs `sha256sum -c checksums.sha256 --ignore-missing`
- **THEN** the command exits 0 and reports the file as OK
