# Forja upstream

| Field | Value |
|---|---|
| Upstream | `storytold/cadcraft` |
| Last observed | `6f0f1fefa81d08031bde56a8b3d5b1556810f397` |
| Last evaluated | `6f0f1fefa81d08031bde56a8b3d5b1556810f397` |
| Last integrated | `6f0f1fefa81d08031bde56a8b3d5b1556810f397` imported with full history on 2026-11-10 |
| Skipped | none |
| Adapted | app icon; every CADCraft name renamed to Forja (`.github/rebrand.toml`); DWG/DXF, MTEXT and native-open fixes (upstream PR #1086); GNOME Wayland window controls (upstream PR #1368) |
| Conflicting | `docs/file-format-parity.md` when merging the PR #1086 work: upstream's text kept |

Forja shares history with upstream, but never merges `upstream/main` directly: upstream still
calls the app CADCraft, and its crates, paths, app id, environment variables and DXF extended data
say so.

## How upstream arrives renamed

`scripts/rebrand.py` renames upstream's names to Forja's, in file contents and in paths, from the
rules in `.github/rebrand.toml` (literal replacements in order, spans to keep such as links and
upstream's repository name, files to skip such as `NOTICE`, and lines that keep Forja's own value
such as the version). A line marked `rebrand: keep` is left alone, for compatibility code that has
to name the old product.

The branch `upstream-forja` mirrors upstream with Forja's names. Each of its commits holds the
rebranded tree of one upstream commit, has the previous mirror commit and that upstream commit as
parents, and names it in an `Upstream-Commit:` trailer. The weekly sync
(`.github/workflows/upstream-sync.yml`) adds a mirror commit for upstream's new head and merges
that, so upstream's changes arrive already speaking Forja, and a conflict is a real one. Files in
`.github/upstream-keep` keep Forja's version.

The first mirror commit is upstream `6f0f1fef` renamed; it was merged with `-s ours` when the
whole repository was renamed, so only later upstream changes come through it.

`python3 scripts/rebrand.py check` lists anything still using an upstream name.

Before 2026-11-10 this repository tracked Open CAD Studio; that tree is kept on the branch
`archive/opencadstudio`.
