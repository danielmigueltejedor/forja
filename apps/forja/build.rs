//! Windows only: embed the app icon and version info into `forja.exe`. Elsewhere a no-op.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/forja.ico");
    println!("cargo:rerun-if-env-changed=FORJA_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=FORJA_BUILD_SHA");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    if std::path::Path::new("../../assets/app-icon/forja.ico").exists() {
        res.set_icon("../../assets/app-icon/forja.ico");
    }
    res.set("ProductName", "Forja")
        .set("FileDescription", "Forja computer-aided design")
        .set("CompanyName", "Nodalia")
        .set("LegalCopyright", "Copyright (c) 2026 ArtCraft Team and the Forja contributors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "forja.exe")
        .set("InternalName", "forja");
    if let Err(e) = res.compile() {
        if std::env::var_os("FORJA_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=forja.exe built without icon/version resources: {e}");
    }
}
