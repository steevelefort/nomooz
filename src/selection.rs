use wayland_client::protocol::wl_output;

use crate::zone::Zone;

pub struct Selection {
    pub selected_column: Option<u32>,        // Main grid column
    pub selected_line: Option<u32>,          // Main grid line
    pub selected_division: Option<u32>,      // Cell subdivision
    pub zones: Vec<Zone>,                    // Subdivision areas with history
    pub output: Option<wl_output::WlOutput>, // We need to know the selected screen
}
