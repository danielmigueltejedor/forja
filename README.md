<div align="center">
  <a href="https://getnodalia.com">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/danielmigueltejedor/nodalia-web/main/assets/brand/nodalia-wordmark-white.svg">
      <img src="https://raw.githubusercontent.com/danielmigueltejedor/nodalia-web/main/assets/brand/nodalia-wordmark-black.svg" width="150" alt="Nodalia">
    </picture>
  </a>
  <br><br>
  <img src="https://raw.githubusercontent.com/danielmigueltejedor/forja/main/assets/app-icon/hicolor/512x512/apps/ai.storyteller.cadcraft.png" width="160" alt="Forja">
  <h1>Forja</h1>
  <p><strong>A CAD application for 2D drafting and DWG/DXF work for the Nodalia suite, built on the CadCraft engine.</strong></p>

  <p>
    <img src="https://img.shields.io/badge/status-early%20stage-orange" alt="Early stage">
    <img src="https://img.shields.io/badge/engine-CadCraft-96501F" alt="CadCraft engine">
    <img src="https://img.shields.io/badge/Rust-native-dea584?logo=rust&logoColor=black" alt="Native Rust">
    <img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-2ea44f" alt="MIT OR Apache-2.0">
  </p>

  <p>
    <a href="#overview">Overview</a> ·
    <a href="#what-you-get-today">Today</a> ·
    <a href="#build-and-run">Build</a> ·
    <a href="#project-status">Status</a> ·
    <a href="#keeping-up-with-upstream">Upstream</a> ·
    <a href="#the-nodalia-suite">Suite</a> ·
    <a href="#license">License</a>
  </p>
</div>

---

## Overview

**Forja** is the CAD application for 2D drafting and DWG/DXF work of [Nodalia](https://getnodalia.com). It is built on the engine of
[CadCraft](https://github.com/storytold/cadcraft), which lives in this repository and is kept up to
date with upstream.

> [!IMPORTANT]
> **Forja is at an early stage.** What runs today is the upstream application.
> Nodalia's own interface is not wired in yet and there are no published builds.

## What you get today

- The CadCraft engine.
- The upstream desktop application. Crate and binary names are still upstream's.
- The engine's own feature description, kept as [`docs/upstream-readme.md`](docs/upstream-readme.md).

## Build and run

You need a recent stable Rust toolchain.

```bash
git clone https://github.com/danielmigueltejedor/forja.git
cd forja
cargo run --release -p cadcraft
```

The crate is still called `cadcraft`; it will be renamed with the product.

## Project status

| Area | State |
|---|---|
| Engine | Imported from `storytold/cadcraft` and tracked in [`docs/upstream.md`](docs/upstream.md) |
| Product name | Pending: crates and binary still carry upstream's name (`cadcraft`) |
| App icon | Done: portrait of Leonardo da Vinci |
| Nodalia interface | Not started |
| Shared interface resources | Planned: the Nodalia apps will share interface components and styles |
| Installable packages | None published |

## Changes on top of upstream

Forja already carries fixes that are not in CadCraft's `main`:

- **DWG/DXF layouts, MTEXT and native open.** The work of upstream pull request
  [#1086](https://github.com/storytold/cadcraft/pull/1086): deferred layouts and native viewport
  references are preserved, first-line MTEXT units are resolved, and dense polyline grips are cached.
- **GNOME Wayland window controls.** Upstream pull request
  [#1368](https://github.com/storytold/cadcraft/pull/1368), still open: window controls without
  bundled fonts.

## Keeping up with upstream

[`docs/upstream.md`](docs/upstream.md) records which upstream commit is integrated. Updates are
merges of upstream's `main`, so Forja's own changes are kept and conflicts are resolved in the open.

## App icon

Nodalia app icons are black-and-white portraits of historical figures over the app colour.
Forja's is Leonardo da Vinci, engineer and master of technical drawing. Provenance and licence are in [`assets/app-icon/`](assets/app-icon/).

## The Nodalia suite

Forja is one of the applications of [Nodalia](https://getnodalia.com), an independent software
project developed in public by [Daniel Miguel Tejedor](https://danielmigueltejedor.com).

## License

`MIT OR Apache-2.0`, as upstream. The licence files and `NOTICE` of CadCraft are kept.
See [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).

## Credits

Forja would not exist without [CadCraft](https://github.com/storytold/cadcraft) and its contributors.
