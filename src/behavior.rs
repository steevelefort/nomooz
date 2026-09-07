use std::error::Error;

use crate::configuration::Configuration;

pub struct Behavior {
    pub auto_click: bool,
}

impl Behavior {
    // Unused Result, but we keep it for later.
    pub fn from_config(config: &Configuration) -> Result<Self, Box<dyn Error>> {
        let auto_click = config.get_bool_or_default("behavior.auto-click", true);
        println!("{:?}", auto_click);

        Ok(Self { auto_click })
    }
}
