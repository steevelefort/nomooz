use smithay_client_toolkit::{
    seat::keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
    shell::WaylandSurface,
};
use wayland_client::{
    Connection, QueueHandle,
    protocol::{wl_keyboard, wl_surface},
};
use xkbcommon_rs::keysym::keysym_to_lower;

use crate::{
    cache::cache_data,
    direction::{Direction, find_output_in_direction},
    labels::Labels,
    main_layer::{DOUBLE_CLICK_WINDOW_MS, MainLayer},
    selection::Selection,
    virtual_pointer::ClickButton,
    zone::Zone,
};

impl MainLayer {
    fn handle_division_selection_and_confirm(&mut self, event: &KeyEvent, click: bool) -> bool {
        for (key_line, first_key) in [16, 30, 44].iter().enumerate() {
            for i in 0..10 {
                if event.raw_code == first_key + i {
                    let choice = key_line as u32 * 10 + i;
                    log::debug!("Division selected: {choice}");
                    // println!("Division selected: {choice}");
                    self.selection[self.current_selection_index].selected_division = Some(choice);
                    if click {
                        log::debug!("Final left click");
                        self.click_button = Some(ClickButton::Left);
                        self.double_click = false;
                        self.exit = true;
                    }
                    return true;
                }
            }
        }
        false
    }

    fn handle_line_selection_and_confirm(&mut self, event: &KeyEvent) -> bool {
        for (key_line, first_key) in [16, 30, 44].iter().enumerate() {
            for i in 0..10 {
                if event.raw_code == first_key + i {
                    let choice = key_line as u32 * 10 + i;
                    log::debug!("Line selected: {choice}");
                    self.selection[self.current_selection_index].selected_line = Some(choice);
                    return true;
                }
            }
        }
        false
    }

    fn handle_column_selection_and_confirm(&mut self, event: &KeyEvent) -> bool {
        for i in 0..10 {
            if event.raw_code == 30 + i {
                log::debug!("Column selected: {i}");
                self.selection[self.current_selection_index].selected_column = Some(i);
                return true;
            }
        }
        false
    }

    fn handle_change_display(&mut self, qh: &QueueHandle<Self>, event: &KeyEvent) {
        let direction = if event.keysym == keysym_to_lower(&self.bindings.left)
            || event.keysym == Keysym::Left
        {
            Some(Direction::Left)
        } else if event.keysym == keysym_to_lower(&self.bindings.right)
            || event.keysym == Keysym::Right
        {
            Some(Direction::Right)
        } else if event.keysym == keysym_to_lower(&self.bindings.down)
            || event.keysym == Keysym::Down
        {
            Some(Direction::Down)
        } else if event.keysym == keysym_to_lower(&self.bindings.up) || event.keysym == Keysym::Up {
            Some(Direction::Up)
        } else {
            None
        };

        log::debug!("Direction requested: {direction:?}");
        if let Some(direction) = direction {
            if let Some(current_output) = self.current_output.clone() {
                if let Some(current_info) = self.output_info(&current_output) {
                    log::debug!("Current output: {current_info:?}");
                    let outputs = self.outputs();
                    log::debug!("Known outputs: {outputs:?}");
                    if let Some(output) =
                        find_output_in_direction(&current_info, &outputs, direction)
                    {
                        log::debug!("Target output found: {output:?}");
                        self.switch_output(qh, &output);
                    } else {
                        log::warn!("No target output found in this direction");
                    }
                }
            } else {
                log::warn!("current_output is None: surface_enter not received yet");
            }
        }
    }

    /// Called each time the repeat timer trigger, while the key is held.
    pub fn handle_key_repeat(&mut self, qh: &QueueHandle<Self>, event: KeyEvent) {
        log::trace!("Key repeat: {event:?} {}", event.raw_code);
        self.handle_key(qh, &event);
    }

    fn handle_cancel_and_confirm(&mut self, event: &KeyEvent) -> bool {
        let cancel_selection = keysym_to_lower(&self.bindings.cancel_selection);
        let selection = &mut self.selection[self.current_selection_index];
        if event.keysym == cancel_selection {
            log::debug!("Undo");
            self.pending_click = None;
            if !selection.zones.is_empty() {
                selection.zones.pop();
                return true;
            } else if !selection.selected_line.is_none() {
                selection.selected_line = None;
                return true;
            } else if !selection.selected_column.is_none() {
                selection.selected_column = None;
                return true;
            }
        }
        false
    }
}

impl KeyboardHandler for MainLayer {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _keysyms: &[Keysym],
    ) {
        if self.layer.wl_surface() == surface {
            self.keyboard_focus = true;
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        if self.layer.wl_surface() == surface {
            self.keyboard_focus = false;
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        log::trace!("Key press: {event:?} {}", event.raw_code);
        self.handle_key(qh, &event);
    }

    // Required by the trait, unused here.
    fn repeat_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        _event: KeyEvent,
    ) {
    }

    // Required by the trait, unused here.
    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _event: KeyEvent,
    ) {
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _serial: u32,
        modifiers: Modifiers,
        _raw_modifiers: RawModifiers,
        layout: u32,
    ) {
        self.modifiers = modifiers;

        // Set the layout for the current loaded keymap
        if let Err(error) = cache_data("layout.dat", layout.to_string()) {
            log::error!("Can’t access cache directory: {}", error);
        }
        self.xkb_parser.set_layout(layout as usize);
        self.labels = Some(Labels::rebuild(&self.xkb_parser));
        self.need_redraw = true;
    }

    fn update_keymap(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        keymap: smithay_client_toolkit::seat::keyboard::Keymap<'_>,
    ) {
        // Load the new provided xkb keymap
        if let Err(error) = cache_data("keymap.xkb", keymap.as_string()) {
            log::error!("Can’t access cache directory: {}", error);
        }
        if let Err(error) = self.xkb_parser.parse_from_string(keymap.as_string()) {
            log::error!("Can’t parse xkb-keymap: {error}");
            return;
        }
        self.labels = Some(Labels::rebuild(&self.xkb_parser));
        self.need_redraw = true;
    }
}

impl MainLayer {
    fn handle_key(&mut self, qh: &QueueHandle<Self>, event: &KeyEvent) {
        let mut need_draw = false;

        if !self.modifiers.ctrl {
            if !self.modifiers.shift {
                // press 'esc' to exit
                if event.keysym == Keysym::Escape {
                    self.exit = true;
                }

                // Manage clicks and double-clicks
                let selection = &self.selection[self.current_selection_index];
                if selection.selected_column.is_some() && selection.selected_line.is_some() {
                    // 50/51/52 = left/middle/right click, double click included.
                    let explicit_button =
                        if event.keysym == keysym_to_lower(&self.bindings.left_click) {
                            Some(ClickButton::Left)
                        } else if event.keysym == keysym_to_lower(&self.bindings.middle_click) {
                            Some(ClickButton::Middle)
                        } else if event.keysym == keysym_to_lower(&self.bindings.right_click) {
                            Some(ClickButton::Right)
                        } else {
                            None
                        };

                    if let Some(button) = explicit_button {
                        if self.selection.len() == 1 {
                            let is_repeat_click =
                                if let Some((pending_button, pending_time)) = self.pending_click {
                                    pending_button == button
                                        && event.time.saturating_sub(pending_time)
                                            <= DOUBLE_CLICK_WINDOW_MS
                                } else {
                                    false
                                };

                            if is_repeat_click {
                                log::debug!("Double click");
                                self.click_button = Some(button);
                                self.double_click = true;
                                self.pending_click = None;
                                self.exit = true;
                            } else {
                                log::debug!("Click pending (waiting for a possible double click)");
                                self.pending_click = Some((button, event.time));
                            }
                        } else {
                            log::debug!("Final click (multi-selection, immediate)");
                            self.click_button = Some(button);
                            self.double_click = false;
                            self.exit = true;
                        }
                    }
                }

                // Second selection enabled on first step only.
                if self.current_selection_index == 0
                    && !selection.selected_line.is_none()
                    && event.keysym == keysym_to_lower(&self.bindings.next_selection)
                {
                    log::debug!("New selection");

                    self.selection.push(Selection {
                        selected_column: None,
                        selected_line: None,
                        selected_division: None,
                        zones: Vec::new(),
                        output: self.current_output.clone(),
                    });
                    self.current_selection_index += 1;
                    need_draw = true;
                }

                // Backspace allow to cancel the last selection step
                if self.handle_cancel_and_confirm(event) {
                    need_draw = true;
                }

                // Are we waiting for a column ?
                if self.selection[self.current_selection_index]
                    .selected_column
                    .is_none()
                {
                    // Column selection
                    if self.handle_column_selection_and_confirm(event) {
                        need_draw = true;
                    }
                } else
                // Are we waiting for a line ?
                if self.selection[self.current_selection_index]
                    .selected_line
                    .is_none()
                {
                    // Line selection
                    if self.handle_line_selection_and_confirm(event) {
                        need_draw = true;
                        // Test: try to move mouse pointer
                        self.move_pointer_to_active_zone(qh);
                    }
                } else
                // Are we waiting for a division ?
                if self.selection[self.current_selection_index]
                    .selected_division
                    .is_none()
                {
                    // Division selection
                    if self.handle_division_selection_and_confirm(event, self.behavior.auto_click) {
                        need_draw = true;
                        // Test: try to move mouse pointer
                        if !self.exit {
                            self.move_pointer_to_active_zone(qh);
                        }
                    }
                } else
                // We are waiting for a more precise zone
                {
                    if event.keysym == keysym_to_lower(&self.bindings.shrink) {
                        let current_zone = if let Some(zone) =
                            self.selection[self.current_selection_index].zones.last()
                        {
                            *zone
                        } else {
                            Zone::from_selection(
                                &self.selection[self.current_selection_index],
                                self.width,
                                self.height,
                            )
                        };

                        if let Some(new_zone) = current_zone.halved() {
                            self.selection[self.current_selection_index]
                                .zones
                                .push(new_zone);

                            need_draw = true;
                        }
                    }

                    // Try to move the current zone (and create it if it doen’t exist
                    let direction = if event.keysym == keysym_to_lower(&self.bindings.left)
                        || event.keysym == Keysym::Left
                    {
                        Some(Direction::Left)
                    } else if event.keysym == keysym_to_lower(&self.bindings.right)
                        || event.keysym == Keysym::Right
                    {
                        Some(Direction::Right)
                    } else if event.keysym == keysym_to_lower(&self.bindings.down)
                        || event.keysym == Keysym::Down
                    {
                        Some(Direction::Down)
                    } else if event.keysym == keysym_to_lower(&self.bindings.up)
                        || event.keysym == Keysym::Up
                    {
                        Some(Direction::Up)
                    } else {
                        None
                    };

                    if let Some(direction) = direction {
                        let current_zone = if let Some(zone) =
                            self.selection[self.current_selection_index].zones.last()
                        {
                            *zone
                        } else {
                            Zone::from_selection(
                                &self.selection[self.current_selection_index],
                                self.width,
                                self.height,
                            )
                        };

                        if let Some(new_zone) =
                            current_zone.moved(direction, self.width, self.height)
                        {
                            self.selection[self.current_selection_index]
                                .zones
                                .push(new_zone);
                        } else {
                            if self.selection[self.current_selection_index]
                                .zones
                                .is_empty()
                            {
                                self.selection[self.current_selection_index]
                                    .zones
                                    .push(current_zone);
                            }
                        }

                        need_draw = true;
                    }

                    // Test: try to move mouse pointer
                    self.move_pointer_to_active_zone(qh);
                }
            } else {
                // Shift pressed !

                let selection = &self.selection[self.current_selection_index];
                if selection.selected_column.is_some()
                    && selection.selected_line.is_some()
                    && selection.selected_division.is_none()
                {
                    // Division selection
                    if self.handle_division_selection_and_confirm(event, !self.behavior.auto_click)
                    {
                        need_draw = true;
                        // Test: try to move mouse pointer
                        if !self.exit {
                            self.move_pointer_to_active_zone(qh);
                        }
                    }
                }
            }
        } else {
            // Ctrl modifier is pressed
            self.handle_change_display(qh, event);
        }

        if need_draw {
            self.need_redraw = true;
        }
    }
}
