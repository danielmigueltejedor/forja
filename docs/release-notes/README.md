# Release notes

Every version has its notes here, written by hand: `docs/release-notes/vX.Y.Z.md`. They are the
text of the GitHub release, so people reading the release page see exactly this file.

## How a release happens

Bump `version` in `[workspace.package]` of `Cargo.toml` and add `docs/release-notes/vX.Y.Z.md` in
the same pull request. When it is merged, **Release from main** (`release-auto.yml`):

1. tags the merge `vX.Y.Z` and creates a draft release whose text is this file;
2. runs the Linux (`release-linux.yml`), macOS (`release-macos.yml`) and Windows (`release-windows.yml`)
   packaging, which attach their files;
3. publishes the release once every package is attached. If one fails, it stays a draft.

A version with no notes file is not released. Nobody pushes a tag by hand.

## What goes in

Write them from the merged pull requests, for someone deciding whether to update: what changed
for them, in their words. No generated lists ("What's Changed", "Full Changelog").

## Format

The **Release notes are written** check (`scripts/check-release-notes.sh`, run on every pull
request by `release-notes.yml` and by `release-auto.yml` before tagging) enforces it.

1. A one-sentence summary in Spanish, then the same in English in italics.
2. `## Español` with `### Novedades` (bullets) and optionally `### Correcciones` and `### Notas`.
3. `## English` with the same sections (`### What's new`, `### Fixes`, `### Notes`).
4. `## Descargas · Downloads`: a table of the real file names of this version (`forja-X.Y.Z-…`),
   plus the checksum files (`SHA256SUMS…`).

A first release uses `### Qué incluye` / `### What's in it` instead of the "what's new" headings.
