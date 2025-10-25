use crate::{APP_INFO, WINDOW};

///Clipboard manager, for getting text from the clipboard or for copying text to the clipboard
pub(crate) struct Clipboard {
    provider: ClipboardProvider,
}

enum ClipboardProvider {
    Arb(arboard::Clipboard),
    Smithay(smithay_clipboard::Clipboard),
}

impl Clipboard {
    ///Creates a new clipboard manager
    pub fn new() -> Result<Self, arboard::Error> {
        #[cfg(target_os = "linux")]
        if APP_INFO.get().unwrap().read().unwrap().is_wayland {
            use wgpu::rwh::HasDisplayHandle;

            let win = WINDOW.get().unwrap();

            let ptr = match win.display_handle().unwrap().as_raw() {
                wgpu::rwh::RawDisplayHandle::Wayland(hndl) => hndl.display,
                _ => unreachable!(),
            };

            log::info!("Smithay!");
            // let wl_display = W ;
            return Ok(Self {
                provider: ClipboardProvider::Smithay(unsafe {
                    smithay_clipboard::Clipboard::new(ptr.as_ptr())
                }),
            });
        }

        Ok(Self {
            provider: ClipboardProvider::Arb(arboard::Clipboard::new()?),
        })
    }

    ///Get text from the clipboard
    pub fn get_clipboard(&mut self) -> String {
        match &mut self.provider {
            ClipboardProvider::Arb(clipboard) => clipboard.get_text().unwrap_or_default(),
            ClipboardProvider::Smithay(clipboard) => clipboard.load().unwrap_or_default(),
        }
    }

    ///Copy text to the clipboard
    pub fn set_clipboard(&mut self, text: String) {
        log::info!("setting clipboard to {text}");
        match &mut self.provider {
            ClipboardProvider::Arb(clipboard) => clipboard.set_text(text).unwrap(),
            ClipboardProvider::Smithay(clipboard) => clipboard.store(text),
        }
    }
}
