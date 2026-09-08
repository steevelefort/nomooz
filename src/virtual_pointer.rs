use std::time::{SystemTime, UNIX_EPOCH};

use smithay_client_toolkit::dispatch2::Dispatch2;
use wayland_client::protocol::{wl_output, wl_pointer, wl_seat};
use wayland_client::{Connection, Proxy, QueueHandle, globals::GlobalList};
use wayland_protocols_wlr::virtual_pointer::v1::client::zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1;
use wayland_protocols_wlr::virtual_pointer::v1::client::zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1;

use crate::main_layer::MainLayer;

/// Linux codes for the mouse buttons.
const BTN_LEFT: u32 = 0x110;
const BTN_RIGHT: u32 = 0x111;
const BTN_MIDDLE: u32 = 0x112;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickButton {
    Left,
    Middle,
    Right,
}

impl ClickButton {
    fn code(&self) -> u32 {
        if *self == ClickButton::Left {
            BTN_LEFT
        } else if *self == ClickButton::Middle {
            BTN_MIDDLE
        } else {
            BTN_RIGHT
        }
    }
}

fn now_millis() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32
}

pub struct VirtualPointerManager {
    manager: ZwlrVirtualPointerManagerV1,
}

impl VirtualPointerManager {
    pub fn bind(globals: &GlobalList, qh: &QueueHandle<MainLayer>) -> Self {
        let manager = globals
            .bind(qh, 1..=2, ())
            .expect("zwlr_virtual_pointer_manager_v1 is not available");
        VirtualPointerManager { manager }
    }

    /// A pointer not linked to one screen.
    pub fn create(&self, qh: &QueueHandle<MainLayer>) -> VirtualPointer {
        let pointer =
            self.manager
                .create_virtual_pointer_with_output(None::<&wl_seat::WlSeat>, None, qh, ());
        VirtualPointer { pointer }
    }

    /// A pointer linked to one screen.
    pub fn create_for_output(
        &self,
        qh: &QueueHandle<MainLayer>,
        output: &wl_output::WlOutput,
    ) -> VirtualPointer {
        let pointer = self.manager.create_virtual_pointer_with_output(
            None::<&wl_seat::WlSeat>,
            Some(output),
            qh,
            (),
        );
        VirtualPointer { pointer }
    }
}

pub struct VirtualPointer {
    pointer: ZwlrVirtualPointerV1,
}

impl VirtualPointer {
    /// x and y go from 0 to the extent.
    pub fn move_absolute(&self, x: u32, y: u32, x_extent: u32, y_extent: u32) {
        self.pointer
            .motion_absolute(now_millis(), x, y, x_extent, y_extent);
        self.pointer.frame();
    }

    pub fn press(&self, button: ClickButton) {
        self.pointer.button(
            now_millis(),
            button.code(),
            wl_pointer::ButtonState::Pressed,
        );
        self.pointer.frame();
    }

    pub fn release(&self, button: ClickButton) {
        self.pointer.button(
            now_millis(),
            button.code(),
            wl_pointer::ButtonState::Released,
        );
        self.pointer.frame();
    }
}

impl Drop for VirtualPointer {
    fn drop(&mut self) {
        self.pointer.destroy();
    }
}

impl Dispatch2<ZwlrVirtualPointerManagerV1, MainLayer> for () {
    fn event(
        &self,
        _state: &mut MainLayer,
        _proxy: &ZwlrVirtualPointerManagerV1,
        _event: <ZwlrVirtualPointerManagerV1 as Proxy>::Event,
        _conn: &Connection,
        _qh: &QueueHandle<MainLayer>,
    ) {
        unreachable!("zwlr_virtual_pointer_manager_v1 has no events")
    }
}

impl Dispatch2<ZwlrVirtualPointerV1, MainLayer> for () {
    fn event(
        &self,
        _state: &mut MainLayer,
        _proxy: &ZwlrVirtualPointerV1,
        _event: <ZwlrVirtualPointerV1 as Proxy>::Event,
        _conn: &Connection,
        _qh: &QueueHandle<MainLayer>,
    ) {
        unreachable!("zwlr_virtual_pointer_v1 has no events")
    }
}
