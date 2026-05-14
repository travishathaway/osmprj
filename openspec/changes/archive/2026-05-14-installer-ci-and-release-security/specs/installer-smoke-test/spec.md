## ADDED Requirements

### Requirement: Smoke test runs on push to main
The CI system SHALL build the installer and verify it installs correctly on every push to the `main` branch, using native GitHub Actions runners for supported platforms.

#### Scenario: Successful install on linux-64
- **WHEN** a commit is pushed to `main`
- **THEN** the workflow builds `osmprj-linux-64-installer.sh`, runs it with `--output-directory /tmp/osmprj-test`, and asserts `osmprj --version` exits 0

#### Scenario: Successful install on osx-arm64
- **WHEN** a commit is pushed to `main`
- **THEN** the workflow builds `osmprj-osx-arm64-installer.sh`, runs it with `--output-directory /tmp/osmprj-test`, and asserts `osmprj --version` exits 0

### Requirement: Smoke test is manually triggerable from any branch
The CI system SHALL allow the smoke test to be triggered manually via `workflow_dispatch` from any branch, enabling developers to test installer changes before merging.

#### Scenario: Manual dispatch from a feature branch
- **WHEN** a developer triggers the workflow manually from a feature branch via the GitHub Actions UI
- **THEN** the smoke test runs against that branch's code with no required inputs

### Requirement: Smoke test failure blocks confidence in release
The smoke test job SHALL exit with a non-zero code if the installer fails to run or the binary is not functional after installation.

#### Scenario: Installer fails to execute
- **WHEN** the installer script exits non-zero
- **THEN** the smoke test job fails and reports the failure

#### Scenario: Binary not functional after install
- **WHEN** `osmprj --version` exits non-zero after installation
- **THEN** the smoke test job fails
