## ADDED Requirements

### Requirement: scripts/install.sh is checked with shellcheck on commit
The pre-commit configuration SHALL include a shellcheck hook that runs against `scripts/install.sh` on every commit.

#### Scenario: shellcheck passes on valid shell script
- **WHEN** a developer commits changes and `scripts/install.sh` has no shellcheck violations
- **THEN** the pre-commit hook exits 0 and the commit proceeds

#### Scenario: shellcheck fails on invalid shell syntax
- **WHEN** a developer commits changes and `scripts/install.sh` contains a shellcheck violation
- **THEN** the pre-commit hook exits non-zero, the commit is blocked, and the violation is reported

### Requirement: shellcheck runs in CI via the existing pre-commit job
The shellcheck hook SHALL be covered by the existing `pre-commit` CI job in `ci.yml` without requiring a new workflow job.

#### Scenario: shellcheck runs in CI on pull requests
- **WHEN** a pull request is opened or updated
- **THEN** the pre-commit CI job runs shellcheck as part of the prek hook suite
