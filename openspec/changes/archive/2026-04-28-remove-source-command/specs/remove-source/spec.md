## ADDED Requirements

### Requirement: Remove command accepts one or more source names
The `osmprj remove` command SHALL accept one or more positional `<source>` arguments identifying sources to remove from the project.

#### Scenario: Single source removal
- **WHEN** user runs `osmprj remove bremen`
- **THEN** the system removes `bremen` from `osmprj.toml`, `osmprj.lock`, and the database

#### Scenario: Multiple source removal
- **WHEN** user runs `osmprj remove berlin hamburg`
- **THEN** the system removes both `berlin` and `hamburg` from all three artefacts in sequence

### Requirement: Source must exist in osmprj.toml
The system SHALL validate that every provided source name exists in the `[sources]` table of `osmprj.toml` before performing any mutations. If any name is not found, the command SHALL exit with a `SourceNotFound` error without modifying any artefact.

#### Scenario: Unknown source name
- **WHEN** user runs `osmprj remove nonexistent`
- **THEN** the system prints a `SourceNotFound` diagnostic and exits with a non-zero code without modifying `osmprj.toml`, `osmprj.lock`, or the database

#### Scenario: Mix of valid and invalid names
- **WHEN** user runs `osmprj remove bremen nonexistent`
- **THEN** the system fails before removing `bremen`, leaving all artefacts unchanged

### Requirement: osmprj.toml entry is removed preserving formatting
The system SHALL remove the named source's entry from the `[sources]` table using `toml_edit`, preserving all other formatting, comments, and key ordering in the file.

#### Scenario: Config file updated
- **WHEN** a source is successfully removed
- **THEN** the source key no longer appears under `[sources]` in `osmprj.toml` and unrelated entries and comments are unchanged

### Requirement: Database schema is dropped
The system SHALL drop the source's PostgreSQL schema (using `DROP SCHEMA IF EXISTS "<schema>" CASCADE`) when a database URL is configured and the connection succeeds.

#### Scenario: Schema dropped successfully
- **WHEN** the database is reachable and the schema exists
- **THEN** the schema and all its objects are dropped and the system prints `Dropped schema '<schema>'`

#### Scenario: Schema already absent
- **WHEN** the database is reachable but the schema does not exist
- **THEN** `DROP SCHEMA IF EXISTS` succeeds silently with no error

#### Scenario: Database unreachable
- **WHEN** the database URL is set but the connection fails
- **THEN** the system prints a warning, skips the schema drop, and continues removing from config and lock file

#### Scenario: No database URL configured
- **WHEN** `database_url` is absent from `osmprj.toml`
- **THEN** the system skips the schema drop without error and prints a hint to configure the URL

### Requirement: osmprj.lock entry is removed
The system SHALL remove the named source's entry from `osmprj.lock` and persist the updated file. If the source has no entry in the lock file (e.g., never synced), the step is silently skipped.

#### Scenario: Lock entry removed
- **WHEN** the source has an entry in `osmprj.lock`
- **THEN** the entry is removed and the file is rewritten

#### Scenario: Source absent from lock file
- **WHEN** the source has no entry in `osmprj.lock`
- **THEN** the lock file is left unchanged without any error

### Requirement: Confirmation prompt before any mutation
Before modifying any artefact, the system SHALL print a summary of all planned actions and prompt the user to confirm with `y/N`. If the user responds with anything other than `y` or `Y`, the command SHALL abort without making any changes.

#### Scenario: User confirms removal
- **WHEN** user runs `osmprj remove bremen` and types `y` at the prompt
- **THEN** the system proceeds to remove the source from all three artefacts

#### Scenario: User declines removal
- **WHEN** user runs `osmprj remove bremen` and types `n` (or presses Enter) at the prompt
- **THEN** the system prints an abort message and exits with code 0 without modifying any artefact

#### Scenario: Summary lists all affected artefacts
- **WHEN** the confirmation prompt is displayed
- **THEN** it lists each source to be removed, the database schema that will be dropped, and notes the action is irreversible

### Requirement: Force flag bypasses confirmation prompt
When `-f` or `--force` is passed, the system SHALL skip the confirmation prompt and proceed directly to removing the named sources.

#### Scenario: Force flag skips prompt
- **WHEN** user runs `osmprj remove --force bremen`
- **THEN** the system removes `bremen` without displaying a confirmation prompt

#### Scenario: Short force flag accepted
- **WHEN** user runs `osmprj remove -f bremen`
- **THEN** the system removes `bremen` without displaying a confirmation prompt

### Requirement: Dry-run mode previews changes without modifying anything
When `--dry-run` is passed, the system SHALL print what would be removed from `osmprj.toml`, `osmprj.lock`, and the database without making any changes. The confirmation prompt SHALL be skipped in dry-run mode because no destructive action is taken.

#### Scenario: Dry-run output
- **WHEN** user runs `osmprj remove --dry-run bremen`
- **THEN** the system prints lines describing each planned action (config removal, lock removal, schema drop) and exits with code 0 without displaying a confirmation prompt or modifying any artefact

### Requirement: Per-step confirmation output
For each completed action the system SHALL print a confirmation line so the user knows what happened.

#### Scenario: Successful removal output
- **WHEN** all three artefacts are updated successfully for a source named `bremen` with schema `bremen`
- **THEN** the system prints at minimum:
  - `Removed [sources.bremen] from osmprj.toml`
  - `Dropped schema 'bremen'`
  - `Removed bremen from osmprj.lock`
