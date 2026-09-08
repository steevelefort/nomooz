use std::path::PathBuf;

use kdl::{KdlDocument, KdlNode, KdlValue};

pub struct Configuration {
    data: KdlDocument,
}

impl Configuration {
    const CONFIG_FILENAME: &str = "config.kdl";

    fn config_dir() -> PathBuf {
        let base = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            format!("{}/.config", std::env::var("HOME").expect("HOME not set"))
        });
        PathBuf::from(base).join(env!("CARGO_PKG_NAME"))
    }

    fn write_and_get_default_config() -> Result<String, std::io::Error> {
        let data = include_str!("../assets/default-config.kdl");
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        std::fs::write(dir.join(Self::CONFIG_FILENAME), data)?;
        Ok(data.to_string())
    }

    // Try to load config file, or create a default one
    fn load_config_data() -> Result<String, std::io::Error> {
        let config_text =
            match std::fs::read_to_string(Self::config_dir().join(Self::CONFIG_FILENAME)) {
                Ok(text) => text,
                Err(_) => Self::write_and_get_default_config()?,
            };
        Ok(config_text)
    }

    pub fn load_config() -> Result<Self, Box<dyn std::error::Error>> {
        let config_text = Self::load_config_data()?;

        Ok(Configuration {
            data: config_text.parse()?,
        })
    }

    fn get_raw(&self, key: &str) -> Result<&KdlValue, Box<dyn std::error::Error>> {
        let keys: Vec<&str> = key.split('.').collect();

        let mut doc = &self.data;
        let mut node: Option<&KdlNode> = None;

        for (index, key) in keys.iter().enumerate() {
            node = doc.get(key);
            let n = node.ok_or(format!("{} Not found", key))?;
            if index < keys.len() - 1 {
                doc = n.children().ok_or(format!("{}: No child", key))?;
            }
        }

        let value = node
            .ok_or("Unexpected node error")?
            .get(0)
            .ok_or("Not value attached")?;

        Ok(value)
    }

    pub fn get_string(&self, key: &str) -> Result<String, Box<dyn std::error::Error>> {
        let value = self.get_raw(key)?.as_string().ok_or("Not a string")?;

        Ok(value.to_string())
    }

    pub fn get_string_or_default(&self, key: &str, default: String) -> String {
        if let Ok(value) = self.get_string(key) {
            value
        } else {
            default
        }
    }

    pub fn get_bool(&self, key: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let value = self.get_raw(key)?.as_bool().ok_or("Not a boolean")?;

        Ok(value)
    }

    pub fn get_bool_or_default(&self, key: &str, default: bool) -> bool {
        if let Ok(value) = self.get_bool(key) {
            value
        } else {
            default
        }
    }
}
