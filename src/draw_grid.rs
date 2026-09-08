use crate::color_rgba::ColorRGBA;
use crate::text_renderer::{TextRenderer, TextStyle};
use crate::zone::Zone;

pub fn draw_grid(
    zone: Zone,
    cells_count_x: u32,
    cells_count_y: u32,
    color: ColorRGBA,
    canvas: &mut [u8],
    canvas_width: usize,
) {
    let x = zone.position.x;
    let y = zone.position.y;
    let width = zone.size.width;
    let height = zone.size.height;

    let fact = (width / cells_count_x) as usize;
    for i in 1..cells_count_x as usize {
        for j in y as usize..(y as usize + height as usize) {
            for k in 0..3usize {
                color.print_on_canvas(x as usize + i * fact + k - 1, j, canvas, canvas_width);
            }
        }
    }

    let fact = (height / cells_count_y) as usize;
    for i in 1..cells_count_y as usize {
        for j in x as usize..(x as usize + width as usize) {
            for k in 0..3usize {
                color.print_on_canvas(j, y as usize + i * fact + k - 1, canvas, canvas_width);
            }
        }
    }
}

pub fn draw_grid_labels(
    zone: Zone,
    labels: &[Vec<String>],
    style: TextStyle,
    text_renderer: &TextRenderer,
    canvas: &mut [u8],
    canvas_width: usize,
) {
    let x = zone.position.x;
    let y = zone.position.y;
    let width = zone.size.width;
    let height = zone.size.height;

    // Grid dimensions come straight from the labels shape.
    let cells_count_y = labels.len() as u32;
    let cells_count_x = labels[0].len() as u32;

    let cell_width = width / cells_count_x;
    let cell_height = height / cells_count_y;

    for (row, row_labels) in labels.iter().enumerate() {
        for (col, label) in row_labels.iter().enumerate() {
            let center_x = x + col as u32 * cell_width + cell_width / 2;
            let center_y = y + row as u32 * cell_height + cell_height / 2;

            text_renderer.print_text(
                label,
                center_x as f32,
                center_y as f32,
                style,
                canvas,
                canvas_width,
            );
        }
    }
}
