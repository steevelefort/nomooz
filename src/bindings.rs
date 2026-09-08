use std::error::Error;

use smithay_client_toolkit::seat::keyboard::Keysym;
use xkbcommon_rs::keysym::keysym_from_name;

use crate::configuration::Configuration;

pub struct Bindings {
    pub left_click: Keysym,
    pub middle_click: Keysym,
    pub right_click: Keysym,
    pub next_selection: Keysym,
    pub cancel_selection: Keysym,
    pub left: Keysym,
    pub down: Keysym,
    pub up: Keysym,
    pub right: Keysym,
    pub shrink: Keysym,
}

impl Bindings {
    pub fn from_config(config: &Configuration) -> Result<Self, Box<dyn Error>> {
        let left_click_setting =
            config.get_string_or_default("binds.left-click", "space".to_string());
        let middle_click_setting =
            config.get_string_or_default("binds.middle-click", "8".to_string());
        let right_click_setting =
            config.get_string_or_default("binds.right-click", "9".to_string());
        let next_selection_setting =
            config.get_string_or_default("binds.next-selection", "Return".to_string());
        let cancel_selection_setting =
            config.get_string_or_default("binds.cancel-selection", "BackSpace".to_string());
        let left_setting = config.get_string_or_default("binds.left", "h".to_string());
        let down_setting = config.get_string_or_default("binds.down", "j".to_string());
        let up_setting = config.get_string_or_default("binds.up", "k".to_string());
        let right_setting = config.get_string_or_default("binds.right", "l".to_string());
        let shrink_setting = config.get_string_or_default("binds.shrink", "i".to_string());

        Ok(Self {
            left_click: keysym_from_name(&left_click_setting, 1)
                .ok_or(format!("Unknown keysym: {left_click_setting}"))?,
            middle_click: keysym_from_name(&middle_click_setting, 1)
                .ok_or(format!("Unknown keysym: {middle_click_setting}"))?,
            right_click: keysym_from_name(&right_click_setting, 1)
                .ok_or(format!("Unknown keysym: {right_click_setting}"))?,
            next_selection: keysym_from_name(&next_selection_setting, 1)
                .ok_or(format!("Unknown keysym: {next_selection_setting}"))?,
            cancel_selection: keysym_from_name(&cancel_selection_setting, 1)
                .ok_or(format!("Unknown keysym: {cancel_selection_setting}"))?,
            left: keysym_from_name(&left_setting, 1)
                .ok_or(format!("Unknown keysym: {left_setting}"))?,
            down: keysym_from_name(&down_setting, 1)
                .ok_or(format!("Unknown keysym: {down_setting}"))?,
            up: keysym_from_name(&up_setting, 1).ok_or(format!("Unknown keysym: {up_setting}"))?,
            right: keysym_from_name(&right_setting, 1)
                .ok_or(format!("Unknown keysym: {right_setting}"))?,
            shrink: keysym_from_name(&shrink_setting, 1)
                .ok_or(format!("Unknown keysym: {shrink_setting}"))?,
        })
    }
}
