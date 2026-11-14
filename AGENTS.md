# Forja: instructions for agents

<!-- Nodalia template: https://github.com/getnodalia/foundation/blob/main/standards/agents.md
     General rules (engineering, CI, security, release, NDL) live in getnodalia/foundation and are
     not repeated here. This file holds only what is particular to this repository. -->

Engineering rules for every Nodalia repository: [getnodalia/foundation](https://github.com/getnodalia/foundation)
([engineering](https://github.com/getnodalia/foundation/blob/main/standards/engineering.md),
[CI](https://github.com/getnodalia/foundation/blob/main/standards/ci.md),
[security](https://github.com/getnodalia/foundation/blob/main/standards/security.md),
[release](https://github.com/getnodalia/foundation/blob/main/standards/release.md),
[NDL](https://github.com/getnodalia/foundation/blob/main/standards/ndl.md)).

## What this is

Forja is Nodalia's computer-aided design and drafting application: 2D drafting and parametric
drawing in pure Rust, with AutoCAD-class commands, DXF and DWG files, and full agent control
(command line, scripts, CLI, control channel, MCP). It runs natively on macOS, Windows, Linux and
FreeBSD, and on the web through WASM. Licence: `MIT OR Apache-2.0`. Its code base began as an
upstream open-source project; those authors are credited in `NOTICE`, which must stay as it is.

## Architecture

| Layer | Path | Responsibility |
|---|---|---|
| L0 | `crates/geom`, `crates/dxf` | f64 geometry, curves, intersections, offsets; standalone DXF tag reader/writer |
| L1 | `crates/color`, `crates/doc` | colour (ACI, true colour, ByLayer/ByBlock); the drawing database |
| L2 | `crates/fonts`, `crates/render`, `crates/constraints` | drafting font and text layout; display lists and CPU rasteriser; parametric solver |
| L3 | `crates/io`, `crates/dwg` | DXF mapping, SVG/PNG/PDF export; DWG through `acadrust` (MPL-2.0, unmodified) |
| L4 | `crates/engine` | sessions, commands, command line and prompts, snaps, selection, undo |
| L5 | `crates/ui-egui`, `crates/mcp` | the swappable egui front end; the MCP server |
| apps | `apps/forja`, `apps/forja-cli`, `apps/forja-web` | desktop app, CLI, web build |

Dependencies point down only, enforced by `cargo xtask layers`. Nothing below L5 depends on
egui, eframe, winit, wgpu or rfd.

**Everything is a command.** User-visible behaviour is a `CommandSpec` in
`crates/engine/src/cmd/*` (id = the AutoCAD command name in lower case, label, menu path,
shortcut, aliases, params doc, `enabled`, JSON `run`, optional interactive prompt machine) plus its
tests. UI-only commands live in `crates/ui-egui/src/menus.rs` (`UI_COMMANDS`). Programmatic calls
(`engine.execute`, MCP, scripts) never open dialogs; only menu and toolbar invocation starts the
interactive prompts. Panels read engine state and act through `app.run`, `app.start` and
`app.cmdline`; colours come from `theme::Tokens`.

## Build and test

```bash
cargo xtask ci                     # what CI runs: fmt, clippy -D warnings, tests, assets, layers, wasm
cargo test -p forja-engine         # fast loop on one crate
cargo run --release -p forja -- --sample --control 7990   # the app with a sample drawing and control channel
forja-cli run --sample --script 'LINE 0,0 10,10\n' --export out.png   # headless
```

Drive the running app with JSON lines on `127.0.0.1:PORT` (`cmdline.input`, `engine.execute`,
`ui.screenshot`); methods are in `crates/ui-egui/src/control.rs`, the protocol in
`docs/control-protocol.md`. For UI work, look at the screenshot; with no presented frame use
`ui.render` or `forja-cli`.

Fonts are an optional build input: `CRAFT_FONTS_DIR` pointing at a checkout of the
`vendor/craft-fonts/main` branch of getnodalia/foundation, embedded by `crates/fonts/build.rs`.
Code and tests work without it. Parallel agents use separate `CARGO_TARGET_DIR`s and delete them
when done.

## Rules

- **Never crash.** No `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!` or
  `unsafe` in non-test code (workspace lints, clippy `-D warnings`). Errors are `Result`. Numbers
  from DXF/DWG files, the command line and MCP/control parameters are hostile: `get()` not `[i]`,
  checked arithmetic, no NaN casts, capped allocations, bounded block recursion (`MAX_BLOCK_DEPTH`).
  `Session::execute` and every interactive step run under `catch_unwind`
  (`crates/engine/src/guard.rs`); a crash fix lands with a regression test
  (`crates/engine/tests/panic_hunt.rs`, `hostile_dxf_does_not_panic`).
- **Clean room.** AutoCAD may only be observed black-box with synthetic drawings; never read,
  disassemble or copy anything from its bundle, and never commit its files or screenshots. Formats
  come from Autodesk's published DXF reference and the Open Design Alliance's public DWG
  specification. Never read or copy GPL/LGPL/AGPL code (LibreDWG, LibreCAD, QCAD, FreeCAD,
  SolveSpace, OpenCASCADE).
- **Assets.** No Autodesk, Adobe or Avid iconography, fonts, hatch patterns, linetypes or
  templates. Icons are drawn in code (`crates/ui-egui/src/icons.rs`), patterns and linetypes are our
  own (`crates/doc/src/library.rs`), and the drafting font is ours (`crates/fonts/src/stroke.rs`).
  Every asset file has a row in `ATTRIBUTION.md` (`cargo xtask assets`).
- **Dependencies.** Git dependencies come only from getnodalia (CI job "Git dependencies come from
  storytold", whose name is kept for branch protection). Licences stay inside Foundation's
  allowlist.
- **Rust only, and never break wasm** (`cargo xtask wasm`).
- The weekly upstream sync keeps the files listed in `.github/upstream-keep`; add any file this
  project owns there.

## NDL

Forja's interface is egui. The Nodalia theme and the shared CAD icons come from
getnodalia/nodalia-design; window controls follow the NDL rules (round controls on Linux only,
native ones on macOS and Windows). What is missing is tracked in `NODALIA_MIGRATION.md`.

## Release

Merging a version bump with its notes (`docs/release-notes/vX.Y.Z.md`, format in that folder's
README) releases it: `release-auto.yml` tags the merge through Foundation's `release-tag`, builds
the packages with `release-linux.yml`, `release-macos.yml` (DMG on a standard macOS runner) and
`release-windows.yml`, and publishes the release. Nobody pushes a tag by hand. Versions are SemVer
in `[workspace.package]`.

## Pull requests

A change is ready when the gate passes, new behaviour has a test that fails without it, UI changes
come with a screenshot taken from the running app, and docs that describe the behaviour are
updated. Pick work from `docs/gaps.md`; name the rows a PR affects in its description instead of
editing the parity docs.

## Known traps

- `mv` and `cp` may be aliased interactive: use `/bin/mv -f` and `/bin/cp -f`.
- Other apps use control ports too: pick a free one.
- About ▸ Contributors is compiled from `contributors/contributors.json` (generated, never edit by
  hand) and `contributors/people.toml` (only a person's own entry, added after asking them).
