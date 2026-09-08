use std::num::NonZeroU32;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_registry,
    output::{OutputHandler, OutputInfo, OutputState},
    reexports::calloop::LoopHandle,
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{Capability, SeatHandler, SeatState, keyboard::Modifiers},
    shell::{
        WaylandSurface,
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
    },
    shm::{
        Shm, ShmHandler,
        slot::{Buffer, SlotPool},
    },
};
use wayland_client::{
    Connection, QueueHandle,
    globals::GlobalList,
    protocol::{wl_keyboard, wl_output, wl_seat, wl_surface},
};

use crate::{
    behavior::Behavior,
    bindings::Bindings,
    dependencies::Dependencies,
    labels::Labels,
    selection::Selection,
    text_renderer::TextRenderer,
    theme::Theme,
    virtual_pointer::{ClickButton, VirtualPointer, VirtualPointerManager},
    wayland_resources::WaylandResources,
    xkb_parser::XkbParser,
    zone::Zone,
};

/// Max delay between two clicks for a double click.
pub const DOUBLE_CLICK_WINDOW_MS: u32 = 400;

pub struct MainLayer {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    loop_handle: LoopHandle<'static, MainLayer>,
    compositor: CompositorState,
    layer_shell: LayerShell,
    shm: Shm,
    virtual_pointer_manager: VirtualPointerManager,
    pub need_redraw: bool,
    pub exit: bool,
    first_configure: bool,
    pub pool: SlotPool,
    pub width: u32,
    pub height: u32,
    pub layer: LayerSurface,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    pub keyboard_focus: bool,
    pub modifiers: Modifiers,
    pub buffer: Option<Buffer>,
    pub click_button: Option<ClickButton>, // Button used for the click.
    pub double_click: bool,                // True if this is a double click.
    pub pending_click: Option<(ClickButton, u32)>, // A click waiting for maybe a second one.
    pub theme: Theme,
    pub bindings: Bindings,
    pub behavior: Behavior,
    pub text_renderer: TextRenderer,
    pub xkb_parser: XkbParser,
    pub selection: Vec<Selection>,
    pub current_selection_index: usize,
    pub labels: Option<Labels>,
    pub current_output: Option<wl_output::WlOutput>,
}

impl MainLayer {
    pub fn new(
        globals: &GlobalList,
        qh: &QueueHandle<Self>,
        wayland_resources: WaylandResources,
        dependencies: Dependencies,
        loop_handle: LoopHandle<'static, MainLayer>,
    ) -> Self {
        let WaylandResources {
            compositor,
            layer_shell,
            shm,
            virtual_pointer_manager,
            pool,
            layer,
        } = wayland_resources;

        let Dependencies {
            text_renderer,
            xkb_parser,
            theme,
            bindings,
            behavior,
        } = dependencies;

        MainLayer {
            registry_state: RegistryState::new(globals),
            seat_state: SeatState::new(globals, qh),
            output_state: OutputState::new(globals, qh),
            loop_handle,
            compositor,
            layer_shell,
            shm,
            virtual_pointer_manager,
            need_redraw: true,
            exit: false,
            first_configure: true,
            pool,
            width: 255,
            height: 255,
            layer,
            keyboard: None,
            keyboard_focus: false,
            modifiers: Modifiers::default(),
            buffer: None,
            click_button: None,
            double_click: false,
            pending_click: None,
            text_renderer,
            xkb_parser,
            selection: vec![Selection {
                selected_column: None,
                selected_line: None,
                selected_division: None,
                zones: Vec::new(),
                output: None,
            }],
            current_selection_index: 0,
            labels: None,
            current_output: None,
            theme,
            bindings,
            behavior,
        }
    }

    pub fn outputs(&self) -> Vec<(wl_output::WlOutput, OutputInfo)> {
        self.output_state
            .outputs()
            .filter_map(|output| self.output_state.info(&output).map(|info| (output, info)))
            .collect()
    }

    /// Returns fresh info for an output
    pub fn output_info(&self, output: &wl_output::WlOutput) -> Option<OutputInfo> {
        self.output_state.info(output)
    }

    /// One pointer is enough for all screens.
    pub fn create_pointer(&self, qh: &QueueHandle<Self>) -> VirtualPointer {
        self.virtual_pointer_manager.create(qh)
    }

    /// Size of all screens together.
    pub fn total_layout_size(&self) -> (i32, i32) {
        let mut total_width = 0;
        let mut total_height = 0;
        for (_, info) in self.outputs() {
            if let Some((x, y)) = info.logical_position
                && let Some((width, height)) = info.logical_size
            {
                if x + width > total_width {
                    total_width = x + width;
                }
                if y + height > total_height {
                    total_height = y + height;
                }
            }
        }
        (total_width, total_height)
    }

    /// Move the overlay: destroy it and create a new one.
    pub fn switch_output(&mut self, qh: &QueueHandle<Self>, output: &wl_output::WlOutput) {
        // Unmap first, so the new surface gets the focus.
        self.layer.wl_surface().attach(None, 0, 0);
        self.layer.commit();

        let surface = self.compositor.create_surface(qh);
        let layer = self.layer_shell.create_layer_surface(
            qh,
            surface,
            Layer::Overlay,
            Some("main_layer"),
            Some(output),
        );
        layer.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
        layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        layer.set_size(0, 0);
        layer.set_exclusive_zone(-1);
        layer.commit();

        // The selection does not move to the new screen.
        self.selection[self.current_selection_index].selected_column = None;
        self.selection[self.current_selection_index].selected_line = None;
        self.selection[self.current_selection_index].zones.clear();

        self.layer = layer;
        self.buffer = None;
        self.first_configure = true;
        self.need_redraw = true;

        // Move the pointer to the center of the new screen.
        if let Some(info) = self.output_info(output)
            && let Some((width, height)) = info.logical_size
            && let Some((pos_x, pos_y)) = info.logical_position
        {
            let (total_width, total_height) = self.total_layout_size();
            let global_x = pos_x + width / 2;
            let global_y = pos_y + height / 2;
            let pointer = self.create_pointer(qh);
            pointer.move_absolute(
                global_x as u32,
                global_y as u32,
                total_width as u32,
                total_height as u32,
            );
        }
    }

    /// Move the pointer to the center of the current selection’s active zone.
    pub fn move_pointer_to_active_zone(&self, qh: &QueueHandle<Self>) {
        let selection = &self.selection[self.current_selection_index];
        if let Some(output) = selection.output.clone()
            && let Some(info) = self.output_info(&output)
            && let Some((pos_x, pos_y)) = info.logical_position
            && let Some((width, height)) = info.logical_size
        {
            let base_zone = match (selection.selected_column, selection.selected_line) {
                (Some(_), Some(_)) if selection.selected_division.is_some() => {
                    Zone::from_selection(selection, width as u32, height as u32)
                }
                (Some(column), Some(line)) => {
                    Zone::from_column_line(column, line, width as u32, height as u32)
                }
                _ => return,
            };
            let active_zone = if let Some(zone) = selection.zones.last() {
                *zone
            } else {
                base_zone
            };

            let global_x =
                pos_x + active_zone.position.x as i32 + (active_zone.size.width / 2) as i32;
            let global_y =
                pos_y + active_zone.position.y as i32 + (active_zone.size.height / 2) as i32;

            let (total_width, total_height) = self.total_layout_size();
            let pointer = self.create_pointer(qh);
            pointer.move_absolute(
                global_x as u32,
                global_y as u32,
                total_width as u32,
                total_height as u32,
            );
        }
    }
}

impl CompositorHandler for MainLayer {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // Required by the trait, unused here.
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        // Required by the trait, unused here.
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        time: u32,
    ) {
        // If no second click comes fast enough, click once.
        if let Some((button, pending_time)) = self.pending_click
            && time.saturating_sub(pending_time) > DOUBLE_CLICK_WINDOW_MS
        {
            self.click_button = Some(button);
            self.double_click = false;
            self.pending_click = None;
            self.exit = true;
        }

        // Don’t draw before the first configure, or it crashes.
        if !self.first_configure {
            self.draw(qh);
        }
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        output: &wl_output::WlOutput,
    ) {
        if self.layer.wl_surface() == surface {
            self.current_output = Some(output.clone());
            // Keep the current selection’s screen up to date.
            self.selection[self.current_selection_index].output = self.current_output.clone();
        }
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Keep the last known screen, don’t reset it here.
    }
}

impl OutputHandler for MainLayer {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    // Required by the trait, unused here.
    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for MainLayer {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.width = NonZeroU32::new(configure.new_size.0).map_or(256, NonZeroU32::get);
        self.height = NonZeroU32::new(configure.new_size.1).map_or(256, NonZeroU32::get);

        // Do the first draw.
        if self.first_configure {
            self.first_configure = false;
            self.draw(qh);
        }
    }
}

impl SeatHandler for MainLayer {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    // Required by the trait, unused here.
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            log::debug!("Set keyboard capability");
            let loop_handle = self.loop_handle.clone();
            let qh_for_repeat = qh.clone();
            let keyboard = self
                .seat_state
                .get_keyboard_with_repeat(
                    qh,
                    &seat,
                    None,
                    loop_handle,
                    Box::new(move |state, _keyboard, event| {
                        state.handle_key_repeat(&qh_for_repeat, event);
                    }),
                )
                .expect("Failed to create keyboard");
            self.keyboard = Some(keyboard);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_some() {
            log::debug!("Unset keyboard capability");
            self.keyboard.take().unwrap().release();
        }
    }

    // Required by the trait, unused here.
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl ShmHandler for MainLayer {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

delegate_registry!(MainLayer);

impl ProvidesRegistryState for MainLayer {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

smithay_client_toolkit::delegate_dispatch2!(MainLayer);
