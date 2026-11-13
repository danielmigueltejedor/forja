//! One background update check at startup, Help ▸ Check for Updates…, and the windows that offer
//! the new release and install it (desktop only; the web build is always the latest). The flow and
//! the windows are `nodalia-update`'s, shared with the other Nodalia apps: it replaces an AppImage
//! or `Forja.app`, installs the MSI on Windows, and points other installs at the release page.

use nodalia_update::egui::{Action, Status, Updater};

use crate::CadApp;

/// What the update check and windows are doing between frames.
pub struct Runtime {
    updater: Updater,
    /// Whether the host asked for a startup check (see [`CadApp::enable_update_check`]).
    enabled: bool,
    /// Help ▸ Check for Updates… was chosen; the next frame's `tick` starts it with the context.
    check_requested: bool,
    /// The current result was already written to the command history.
    result_echoed: bool,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            updater: Updater::new(crate::UPDATE_APP).with_accent([0xbe, 0x7c, 0x4c]),
            enabled: false,
            check_requested: false,
            result_echoed: false,
        }
    }
}

impl CadApp {
    /// Turn on the startup update check (the desktop host does; tests and the web build do not).
    pub fn enable_update_check(&mut self) {
        self.updates_rt.enabled = true;
    }
}

/// Help ▸ Check for Updates…: check now, and show the result whatever it is.
pub fn check_now(app: &mut CadApp) {
    app.updates_rt.check_requested = true;
    app.updates_rt.result_echoed = false;
}

/// Start the checks and collect the answers.
pub fn tick(app: &mut CadApp, ctx: &egui::Context) {
    let rt = &mut app.updates_rt;
    if std::mem::take(&mut rt.check_requested) {
        rt.updater.check_now(ctx);
    } else if rt.enabled {
        rt.updater.check_at_start(ctx);
    }
    let action = rt.updater.poll();
    let outcome = match rt.updater.status() {
        Status::UpToDate => Some(crate::tl!("Forja is up to date.")),
        Status::CheckFailed(_) => Some(crate::tl!("Could not check for updates.")),
        _ => None,
    };
    if let Some(message) = outcome
        && !std::mem::replace(&mut rt.result_echoed, true)
    {
        app.session.echo(message);
    }
    act(ctx, action);
}

/// Draw the update windows while there is one and act on what they ask for.
pub fn show(app: &mut CadApp, ctx: &egui::Context) {
    // An install waits until no drawing holds unsaved changes.
    let clean = !app.session.docs.iter().any(|d| d.is_dirty());
    let language = nodalia_update::Language::from_code(app.language());
    let action = app.updates_rt.updater.show(ctx, language, clean);
    act(ctx, action);
}

fn act(ctx: &egui::Context, action: Action) {
    match action {
        Action::OpenUrl(url) => ctx.open_url(egui::OpenUrl::new_tab(url)),
        Action::Exit => {
            // The new version is starting, or a helper waits for this process to end. The install
            // only ran once nothing was unsaved, so the close goes through; exit anyway if the
            // window has not closed soon.
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_secs(5));
                std::process::exit(0);
            });
        }
        Action::None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_releases_are_forjas_own() {
        assert_eq!(crate::UPDATE_APP.repo, "getnodalia/forja");
        assert_eq!(crate::UPDATE_APP.bundle, "Forja.app");
        assert_eq!(crate::UPDATE_APP.version, env!("CARGO_PKG_VERSION"));
    }

    /// The names Forja's release workflows give its packages, and the systems whose updater must
    /// find each one. A rename in `packaging/` that the updater does not know about would leave
    /// those users on the old version without a word, so it fails here instead.
    #[test]
    fn every_published_package_is_found_by_the_updater_it_is_for() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packaging");
        let read = |path: &str| std::fs::read_to_string(format!("{root}/{path}")).unwrap();
        let macos = read("macos/package.sh");
        assert!(macos.contains(r#"APP="$WORK/Forja.app""#), "the bundle the updater swaps");
        assert!(macos.contains(r#"DMG="$DIST/forja-$VERSION-macos-$ARCH.dmg""#));
        assert!(macos.contains("ARCH=universal"));
        let linux = read("linux/package.sh");
        assert!(linux.contains(r#"BASENAME="forja-$VERSION-linux-$ARCH""#));
        assert!(linux.contains(r#"$BASENAME.AppImage"#));
        let windows = read("windows/package.ps1");
        assert!(windows.contains(r#""forja-$Version-windows-$Arch.msi""#));
        let wxs = read("windows/forja.wxs");
        assert!(wxs.contains(r#"Key="Software\Forja" Name="InstallDir" Value="[INSTALLFOLDER]""#));

        for (package, systems) in [
            ("forja-1.0.0-macos-universal.dmg", &[("macos", "aarch64"), ("macos", "x86_64")][..]),
            ("forja-1.0.0-linux-x86_64.AppImage", &[("linux", "x86_64")]),
            ("forja-1.0.0-linux-aarch64.AppImage", &[("linux", "aarch64")]),
            ("forja-1.0.0-windows-x64.msi", &[("windows", "x86_64")]),
            ("forja-1.0.0-windows-arm64.msi", &[("windows", "aarch64")]),
        ] {
            let found = nodalia_update::systems_for_package(package);
            for system in systems {
                assert!(found.contains(system), "{package} is not offered to {system:?}");
            }
        }
    }

    #[test]
    fn a_check_at_start_is_silent_until_it_finds_something() {
        let mut rt = Runtime::default();
        assert!(!rt.updater.is_open());
        rt.updater.set_status(Status::Checking { user: false });
        assert!(!rt.updater.is_open());
        rt.updater.set_status(Status::Checking { user: true });
        assert!(rt.updater.is_open());
    }
}
