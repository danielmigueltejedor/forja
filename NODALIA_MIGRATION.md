# Nodalia migration: Forja

Category: priority app.
Status: in progress, as of 2026-11-13. Evidence: forja#22 (Cargo dependencies from
getnodalia/foundation) and this pull request (CI on Foundation's reusable workflows).

## Current state

Pure-Rust CAD app (egui), MIT OR Apache-2.0. Cargo git dependencies all come from
getnodalia/foundation; the "Git dependencies come from storytold" job (name kept for branch
protection) rejects any other owner. The gate, ast-grep, FreeBSD, Nix, packaging lint and AGENTS.md
checks call getnodalia/foundation workflows pinned by commit. The macOS DMG is built by
`macos.yml` on a standard runner.

## Desired state

No workflow, dependency or document depends on storytold; Forja has its own release workflow;
the interface uses Nodalia Design throughout.

## Inherited dependencies still pending

| Dependency | Where | Replacement | Blocked on |
|---|---|---|---|
| craft-common-actions `release.yml`, `refresh-contributors.yml`, `resolve-version`, `draft-release` | `.github/workflows/release.yml` (a push to `release`) | `release-auto.yml`: Foundation's `release-tag`, Forja's `release-linux`, `macos` and `release-windows`, then `release-publish` | its first release; then remove `release.yml` |
| craft-common-actions `windows-arm64.yml` | `.github/workflows/windows-arm64.yml` | a Foundation workflow | writing it |
| Weekly sync from the upstream project | `.github/workflows/upstream-sync.yml` | none: intended, renames upstream names | — |

## Technical debt

- Internal names inherited from upstream still appear in some crates and docs; the upstream sync
  renames them as it merges.
- SonarCloud analysis was removed with this migration; CodeQL and Scorecard remain.

## Reusable Nodalia components

getnodalia/foundation (vendored crates, fonts, reusable workflows); getnodalia/nodalia-design
(theme, shared CAD icons, `nodalia-update`).

## NDL work pending

Check every panel and dialog against Foundation's `standards/ndl.md`: hamburger menu with About,
Preferences and Keyboard shortcuts; theme selector only in Preferences; NDL window controls on
Linux only.

## Risks and priorities

1. Release: `release-auto.yml` replaces the inherited workflow, but has not shipped a release yet.
   The inherited one also refreshed the contributor credits, which nothing does now.
2. Windows ARM64 packaging still runs the inherited workflow.

## Recommended order

1. Ship one release through `release-auto.yml`, then delete `release.yml`.
2. Windows ARM64 workflow in Foundation.
3. NDL review of the interface.

## What the first agent should do

1. Read AGENTS.md and this file.
2. Pick the first item of "Recommended order".
