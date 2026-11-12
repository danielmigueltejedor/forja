//! Host modes: a page that runs Forja opens drawings in it and stores what it saves instead
//! of the browser downloading. The messages and the two transports are `craft-web-host`'s
//! (storytold/craft-libs), the same contract as the other Craft apps' bridges, with the
//! `forja:` prefix. This module is Forja's part of it.
//!
//! - `?host=parent` (an `<iframe>`, such as a Nextcloud app) talks to the same-origin parent over
//!   `postMessage`; `data-host="page"` on the canvas (the page's own document, no iframe) uses
//!   DOM `CustomEvent`s on the canvas instead. `?host=parent` wins when both are present. With no
//!   parent, or a parent on another origin, there is no bridge: the normal app with downloads.
//! - `forja:ready` once the app is listening; send a drawing now. Forja reads bytes, so it
//!   says `file: false` and a host sends `bytes`.
//! - `forja:open` (from the host): `{ name, bytes }`. It opens like a file picked with
//!   File › Open or dropped on the window; an oversized drawing is refused before it is copied.
//! - `forja:save` (to the host): `{ name, saveAs, bytes }` for every file Forja would
//!   otherwise download (SAVE, SAVEAS, exports): it replaces the engine's delivery hook. The host
//!   stores the file and reports failures itself. Forja doesn't tell the host which saves
//!   make a file the document, so `saveAs` is always `false`.
//! - `forja:dirty` whenever unsaved changes appear or go away, `forja:failed` when the app
//!   couldn't start.
//!
//! In both host modes the app starts on the Start tab rather than an empty Drawing1, and the
//! host guards leaving the page.

use std::cell::RefCell;
use std::rc::Rc;

use craft_web_host::web::{FileData, Host, Message};
use forja_ui_egui::CadApp;

/// The app id in the host messages (`forja:ready`).
const APP: &str = "forja";

/// The name a drawing gets when the host sent none (the crate says `document` for a blank name).
const DEFAULT_NAME: &str = "Drawing.dxf";

thread_local! {
    /// The bridge [`deliver`] posts through (the engine's hook is a plain `fn`).
    static BRIDGE: RefCell<Option<Bridge>> = const { RefCell::new(None) };
}

#[derive(Clone)]
pub struct Bridge {
    host: Rc<Host>,
}

impl Bridge {
    /// The bridge for this page, or `None` when Forja isn't hosted: no `?host=parent` and no
    /// `data-host="page"` on the canvas, an unframed `?host=parent` page (it stays the normal app
    /// with downloads), or a parent on another origin.
    pub fn connect(canvas: &web_sys::HtmlCanvasElement) -> Option<Self> {
        Some(Self { host: Rc::new(Host::connect(APP, canvas)?) })
    }

    /// Make this the bridge [`deliver`] uses.
    pub fn install(&self) {
        BRIDGE.with(|b| *b.borrow_mut() = Some(self.clone()));
    }

    pub fn post_ready(&self) {
        let _ = self.host.post_ready(env!("CARGO_PKG_VERSION"), false);
    }

    pub fn post_failed(&self, error: &str) {
        let _ = self.host.post_failed(error, None);
    }

    /// Hear drawings from the host; they arrive through the upload inbox.
    pub fn listen(&self, inbox: crate::upload::Inbox) {
        self.host.listen(move |message| {
            if let Message::Open { name, data } = message {
                receive_open(&name, &data, &inbox);
            }
        });
    }

    /// After each logic callback (including a hidden tab's): tell the host when unsaved changes
    /// appear or go away. The crate does not repeat a state it already reported, so this is
    /// idempotent.
    pub fn report_dirty(&self, app: &CadApp) {
        self.host.post_dirty(app.session.docs.iter().any(|d| d.is_dirty()));
    }

    fn post_save(&self, name: &str, bytes: &[u8]) -> Result<(), String> {
        self.host.post_save(name, bytes, None, false)
    }
}

/// The engine's delivery hook in host mode ([`forja_engine::cmd::file::set_deliver`]): hand
/// the file to the host page. `Ok` once it was sent; the engine then counts a drawing as saved.
pub fn deliver(name: &str, _mime: &str, bytes: &[u8]) -> Result<(), String> {
    let bridge = BRIDGE.with(|b| b.borrow().clone()).ok_or("the page that embeds Forja is gone")?;
    bridge.post_save(name, bytes)
}

/// The `forja:open` payload, `{ name, bytes }` where `bytes` holds a drawing: it opens like a
/// file picked with File › Open or dropped on the window. The crate has cleaned the name (and cut
/// it to 255 characters); a blank one comes as `document` and becomes `Drawing.dxf`. An oversized
/// drawing is refused before it is copied.
fn receive_open(name: &str, data: &FileData, inbox: &crate::upload::Inbox) {
    let name = if name == "document" { DEFAULT_NAME } else { name }.to_string();
    let FileData::Bytes(array) = data else {
        inbox.refuse(format!("Open: {name} came as a File; Forja reads bytes"));
        return;
    };
    // Refuse an oversized drawing before copying it into the app's memory (the data's own length).
    if let Some(e) = CadApp::open_size_error(&name, data.size() as u64) {
        inbox.refuse(e);
        return;
    }
    inbox.receive(name, array.to_vec());
}
