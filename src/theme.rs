use std::error::Error;

use crate::{color_rgba::ColorRGBA, configuration::Configuration};

pub struct Theme {
    pub font: String,
    pub text_color: ColorRGBA,
    pub text_background_color: ColorRGBA,
    pub grid_lines_color: ColorRGBA,
    pub grid_cells_color: ColorRGBA,
    pub background_color: ColorRGBA,
}

impl Theme {
    pub fn from_config(config: &Configuration) -> Result<Self, Box<dyn Error>> {
        let font = config.get_string("appearance.font")?;
        let text_color = ColorRGBA::from_hex_string(&config.get_string("appearance.text-color")?)?;
        let text_background_color =
            ColorRGBA::from_hex_string(&config.get_string("appearance.text-background-color")?)?;
        let background_color =
            ColorRGBA::from_hex_string(&config.get_string("appearance.background-color")?)?;
        let grid_lines_color =
            ColorRGBA::from_hex_string(&config.get_string("appearance.grid-lines-color")?)?;
        let grid_cells_color =
            ColorRGBA::from_hex_string(&config.get_string("appearance.grid-cells-color")?)?;

        Ok(Self {
            font,
            text_color,
            text_background_color,
            grid_lines_color,
            grid_cells_color,
            background_color,
        })
    }
}
