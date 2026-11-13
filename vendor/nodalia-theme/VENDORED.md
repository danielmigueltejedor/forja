# nodalia-theme (vendored)

The Rust crate of the Nodalia Design Language from
[getnodalia/nodalia-design](https://github.com/getnodalia/nodalia-design)
(MIT, see `LICENSE`). The crate sources and manifest follow
`packages/egui/crate` at commit `d709c2565769a46e40c20906995515081f95d3f0`, with one exception:
`src/icons/data.rs` stays at the previous reviewed icon catalog,
`b2feb66c7f7b516613c4f2c74091baddb5a50f14`. It already includes the same four
window icons. Update the broader catalog when Forja needs new or changed icons.

The local path keeps the existing release dependency layout.
