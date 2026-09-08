use std::str::from_utf8;

#[derive(Debug, Clone, Copy)]
pub struct ColorRGBA {
    given_r: u8,
    given_g: u8,
    given_b: u8,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl ColorRGBA {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        let alpha_factor = a as f32 / 255.0;
        let rendered_r = (r as f32 * alpha_factor) as u8;
        let rendered_g = (g as f32 * alpha_factor) as u8;
        let rendered_b = (b as f32 * alpha_factor) as u8;

        ColorRGBA {
            given_r: r,
            given_g: g,
            given_b: b,
            r: rendered_r,
            g: rendered_g,
            b: rendered_b,
            a,
        }
    }

    pub fn from_hex_string(value: &str) -> Result<ColorRGBA, Box<dyn std::error::Error>> {
        if value.starts_with("#") && value.len() == 9 {
            let value = &value[1..];
            let mut iter = value.as_bytes().chunks(2);
            let r = u8::from_str_radix(from_utf8(iter.next().ok_or("Bad format")?)?, 16)?;
            let g = u8::from_str_radix(from_utf8(iter.next().ok_or("Bad format")?)?, 16)?;
            let b = u8::from_str_radix(from_utf8(iter.next().ok_or("Bad format")?)?, 16)?;
            let a = u8::from_str_radix(from_utf8(iter.next().ok_or("Bad format")?)?, 16)?;

            Ok(ColorRGBA::new(r, g, b, a))
        } else {
            Err("Invalid color format".into())
        }
    }

    pub fn print_on_canvas(&self, x: usize, y: usize, canvas: &mut [u8], canvas_width: usize) {
        // Skip if outside the canvas, to avoid a crash.
        if x >= canvas_width {
            return;
        }
        // Never panic here, even with a huge x or y.
        let offset = canvas_width
            .saturating_mul(y)
            .saturating_add(x)
            .saturating_mul(4);
        if canvas.len() < 4 || offset > canvas.len() - 4 {
            return;
        }
        let target: &mut [u8; 4] = (&mut canvas[offset..offset + 4]).try_into().unwrap();

        // We must mix the color with the background
        // Get initial background values
        let dst_b = target[0] as f32;
        let dst_g = target[1] as f32;
        let dst_r = target[2] as f32;
        let dst_a = target[3] as f32;

        let src_a = self.a as f32;

        // Background part to keep
        let inv_alpha = 1.0 - src_a / 255.0;

        // Set the color and add the remaining backgroud
        let out_r = self.r as f32 + dst_r * inv_alpha;
        let out_g = self.g as f32 + dst_g * inv_alpha;
        let out_b = self.b as f32 + dst_b * inv_alpha;
        let out_a = src_a + dst_a * inv_alpha;

        *target = [out_b as u8, out_g as u8, out_r as u8, out_a as u8];
    }

    pub fn set_alpha(&mut self, a: u8) {
        let alpha_factor = a as f32 / 255.0;
        let rendered_r = (self.given_r as f32 * alpha_factor) as u8;
        let rendered_g = (self.given_g as f32 * alpha_factor) as u8;
        let rendered_b = (self.given_b as f32 * alpha_factor) as u8;
        self.a = a;
        self.r = rendered_r;
        self.g = rendered_g;
        self.b = rendered_b;
    }
}

impl From<ColorRGBA> for u32 {
    fn from(value: ColorRGBA) -> Self {
        ((value.a as u32) << 24)
            + ((value.r as u32) << 16)
            + ((value.g as u32) << 8)
            + value.b as u32
    }
}

impl From<ColorRGBA> for [u8; 4] {
    fn from(value: ColorRGBA) -> Self {
        [value.b, value.g, value.r, value.a]
    }
}
