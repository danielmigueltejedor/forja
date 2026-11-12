# nodalia-theme (vendored)

The Rust crate of the Nodalia Design Language, copied unchanged from
[danielmigueltejedor/nodalia-design](https://github.com/danielmigueltejedor/nodalia-design)
`packages/egui/crate` at commit `b2feb66c7f7b516613c4f2c74091baddb5a50f14` (MIT, see `LICENSE`).

It is vendored rather than a git dependency because the shared release workflow
(`storytold/craft-common-actions`) only builds git dependencies from github.com/storytold.
To update it, copy `packages/egui/crate/src` and `Cargo.toml` from a newer nodalia-design commit
and change the commit above.
