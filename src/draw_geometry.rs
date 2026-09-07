use crate::color_rgba::ColorRGBA;
use crate::zone::Zone;

// Quickly draw a filled rectangle.
// It completely override pixels and do not manage transparency.
pub fn draw_filled_rectangle(zone: Zone, color: ColorRGBA, canvas: &mut [u8], canvas_width: usize) {
    let (x, y, width, height) = (
        zone.position.x as usize,
        zone.position.y as usize,
        zone.size.width as usize,
        zone.size.height as usize,
    );
    for j in y..y + height {
        // Compute start and end of each line
        let start = (canvas_width * j + x) * 4;
        let end = start + (width * 4);

        // Fill the line
        canvas[start..end]
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .for_each(|array| {
                *array = color.into();
            })
    }
}
