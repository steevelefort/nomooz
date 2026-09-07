use xkbcommon_rs::{Context, Keymap, KeymapFormat, error::keymap::KeymapCompileError};

const XKB_KEYCODE_OFFSET: u32 = 8; // Xkb keycodes are shifted by 8

pub struct XkbParser {
    context: Context,
    keymap: Option<Keymap>,
    layout: usize,
}

impl XkbParser {
    pub fn new() -> Self {
        let context = Context::new(0).unwrap();
        XkbParser {
            context,
            keymap: None,
            layout: 0,
        }
    }

    // Try to load xkb keymap
    pub fn parse_from_string(&mut self, string: String) -> Result<(), KeymapCompileError> {
        let tmp_context = self.context.clone(); // We need a context, not a reference
        let keymap = Keymap::new_from_string(tmp_context, &string, KeymapFormat::TextV1, 0)?;
        self.keymap = Some(keymap);
        Ok(())
    }

    // Returns the character associated to a key code
    pub fn key_code_to_char(&self, key_code: u32) -> char {
        if let Some(keymap) = &self.keymap {
            let code = key_code + XKB_KEYCODE_OFFSET;
            let syms = keymap.key_get_syms_by_level(code, self.layout, 0).unwrap();
            syms.first().unwrap().key_char().unwrap_or('*')
        } else {
            '?'
        }
    }

    // Set the current layout (xkb keymap can contain several)
    pub fn set_layout(&mut self, layout: usize) {
        log::debug!("selected layout {layout}");
        self.layout = layout;
    }
}
