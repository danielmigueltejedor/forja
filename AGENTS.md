# Agent instructions — forja

## Repository-specific deterministic quality contract (2026-10-26)

**Project:** Rust CAD/editor with Trunk web app and native dependencies.

**Engineering requirements:** Protect CAD geometry invariants, file interchange, unit conversions and custom paper/theme formatting; format before PR and compile components with available dependencies.

**Verification commands:** cargo fmt --all -- --check; cargo metadata --locked --no-deps --format-version 1; targeted cargo test.

Before completion, run relevant checks and report actual outputs, including any failures. Add regression tests for changed behavior, edge cases and error paths. Never weaken/remove/skip tests, suppress failures or lower CI thresholds to force a pass. Changes to `QUALITY_GATES.md`, workflows or these rules need human review. Configure the `quality` status as a required check in branch protection. If tooling is unavailable, report it instead of claiming checks passed. Preserve existing project instructions.
