use std::{thread, time::Duration};

use wayland_client::{Connection, QueueHandle};

use crate::{
    main_layer::MainLayer,
    virtual_pointer::{ClickButton, VirtualPointer},
    zone::Zone,
};

/// Distance to count as "near" a screen edge.
const EDGE_MARGIN: i32 = 40;

/// Time to hold the button before release.
const CLICK_HOLD_MS: u64 = 120;

/// Wait time between the two clicks of a double click.
const DOUBLE_CLICK_GAP_MS: u64 = 40;

/// True if (x, y) is near a screen edge.
fn is_near_output_edge(x: i32, y: i32, outputs: &[(i32, i32, i32, i32)], margin: i32) -> bool {
    for &(output_x, output_y, width, height) in outputs {
        let inside =
            x >= output_x && x < output_x + width && y >= output_y && y < output_y + height;
        if inside {
            let near_left = x - output_x <= margin;
            let near_right = (output_x + width) - x <= margin;
            let near_top = y - output_y <= margin;
            let near_bottom = (output_y + height) - y <= margin;
            return near_left || near_right || near_top || near_bottom;
        }
    }
    false
}

/// Move step by step, faster near screen edges.
fn stepped_absolute_move(
    conn: &Connection,
    pointer: &VirtualPointer,
    from: (u32, u32),
    to: (u32, u32),
    extent: (u32, u32),
    outputs: &[(i32, i32, i32, i32)],
) {
    let steps = 30;
    for step in 1..=steps {
        let x = from.0 as f64 + (to.0 as f64 - from.0 as f64) * step as f64 / steps as f64;
        let y = from.1 as f64 + (to.1 as f64 - from.1 as f64) * step as f64 / steps as f64;
        let x = x.round() as u32;
        let y = y.round() as u32;
        pointer.move_absolute(x, y, extent.0, extent.1);
        let _ = conn.flush();
        if !is_near_output_edge(x as i32, y as i32, outputs, EDGE_MARGIN) {
            thread::sleep(Duration::from_millis(5));
        }
    }
}

/// Press and release again, for the second click.
fn send_double_click(pointer: &VirtualPointer, conn: &Connection, button: ClickButton) {
    thread::sleep(Duration::from_millis(DOUBLE_CLICK_GAP_MS));
    pointer.press(button);
    let _ = conn.flush();
    thread::sleep(Duration::from_millis(DOUBLE_CLICK_GAP_MS));
    pointer.release(button);
    let _ = conn.flush();
}

/// Do the click chosen by the user: press, move if needed, release.
pub fn execute_click(main_layer: &MainLayer, qh: &QueueHandle<MainLayer>, conn: &Connection) {
    let no_selection =
        main_layer.selection.len() == 1 && main_layer.selection[0].selected_column.is_none();

    if no_selection {
        let Some(button) = main_layer.click_button else {
            // Nothing selected and no click requested: nothing to do.
            return;
        };

        // No selection: click where the pointer already is.
        let pointer = main_layer.create_pointer(qh);
        pointer.press(button);
        let _ = conn.flush();
        // Some apps need a short hold to detect the click.
        thread::sleep(Duration::from_millis(CLICK_HOLD_MS));
        pointer.release(button);
        let _ = conn.flush();

        if main_layer.double_click {
            send_double_click(&pointer, conn, button);
        }

        return;
    }

    let pointer = main_layer.create_pointer(qh);
    let (total_width, total_height) = main_layer.total_layout_size();
    let extent = (total_width as u32, total_height as u32);
    let mut previous_position = (extent.0 / 2, extent.1 / 2);
    let output_rects: Vec<(i32, i32, i32, i32)> = main_layer
        .outputs()
        .into_iter()
        .filter_map(|(_, info)| {
            let position = info.logical_position?;
            let size = info.logical_size?;
            Some((position.0, position.1, size.0, size.1))
        })
        .collect();

    for (index, selection) in main_layer.selection.iter().enumerate() {
        if let Some(output) = selection.output.clone()
            && let Some(info) = main_layer.output_info(&output)
            && let (Some((pos_x, pos_y)), Some((width, height))) =
                (info.logical_position, info.logical_size)
        {
            // Use this selection’s own screen size.
            let base_zone = match (selection.selected_column, selection.selected_line) {
                (Some(_), Some(_)) if selection.selected_division.is_some() => {
                    Zone::from_selection(selection, width as u32, height as u32)
                }
                (Some(column), Some(line)) => {
                    // No division selected : click the middle of the cell.
                    Zone::from_column_line(column, line, width as u32, height as u32)
                }
                _ => continue,
            };
            let active_zone = if let Some(zone) = selection.zones.last() {
                *zone
            } else {
                base_zone
            };

            // Click the center of the active zone.
            let (local_x, local_y) = (
                active_zone.position.x + active_zone.size.width / 2,
                active_zone.position.y + active_zone.size.height / 2,
            );

            // Convert to global coordinates.
            let global_x = (pos_x + local_x as i32) as u32;
            let global_y = (pos_y + local_y as i32) as u32;

            log::debug!("Moving");
            stepped_absolute_move(
                conn,
                &pointer,
                previous_position,
                (global_x, global_y),
                extent,
                &output_rects,
            );
            previous_position = (global_x, global_y);

            if index == 0
                && let Some(button) = main_layer.click_button
            {
                log::debug!("Button held down");
                pointer.press(button);
                let _ = conn.flush();
            }
        }
    }

    let Some(button) = main_layer.click_button else {
        // No click requested (e.g. Esc): the pointer was moved, nothing else to do.
        return;
    };

    // Wait a bit before release, like a real click.
    thread::sleep(Duration::from_millis(CLICK_HOLD_MS));
    log::debug!("Click released");
    pointer.release(button);
    let _ = conn.flush();

    if main_layer.double_click {
        log::debug!("Second click (double click)");
        send_double_click(&pointer, conn, button);
    }
}
