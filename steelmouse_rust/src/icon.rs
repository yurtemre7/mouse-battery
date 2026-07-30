use image::{Rgba, RgbaImage};
use tray_icon::Icon;
use crate::config::DisplayMode;

const ICON_SIZE: u32 = 32;

pub fn create_tray_icon(
    level: Option<u8>,
    is_charging: bool,
    display_mode: DisplayMode,
) -> Icon {
    let mut img = RgbaImage::from_pixel(ICON_SIZE, ICON_SIZE, Rgba([0, 0, 0, 255]));

    if let Some(pct) = level {
        let color = if is_charging {
            Rgba([255, 165, 0, 255]) // Orange (Charging)
        } else if pct < 20 {
            Rgba([255, 69, 58, 255]) // Red (<20%)
        } else if pct < 50 {
            Rgba([255, 204, 0, 255]) // Yellow (<50%)
        } else {
            Rgba([52, 199, 89, 255]) // Green (>=50%)
        };

        match display_mode {
            DisplayMode::Hover => {
                draw_battery_fill(&mut img, pct, color);
            }
            DisplayMode::Icon => {
                draw_percentage_text(&mut img, pct, color);
            }
        }
    } else {
        draw_error_icon(&mut img);
    }

    make_background_transparent(&mut img);

    let width = img.width();
    let height = img.height();
    let raw_rgba = img.into_raw();

    Icon::from_rgba(raw_rgba, width, height).expect("Failed to build tray icon")
}

fn draw_battery_fill(img: &mut RgbaImage, level: u8, color: Rgba<u8>) {
    let capped_level = level.min(100) as f32;
    let fill_height = ((ICON_SIZE as f32) * (capped_level / 100.0)).round() as u32;

    for y in (ICON_SIZE - fill_height)..ICON_SIZE {
        for x in 0..ICON_SIZE {
            img.put_pixel(x, y, color);
        }
    }
}

// Crisp 5x7 bitmap font for numbers 0-9
const DIGITS_5X7: [&[u8; 7]; 10] = [
    &[0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110], // 0
    &[0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110], // 1
    &[0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111], // 2
    &[0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110], // 3
    &[0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010], // 4
    &[0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110], // 5
    &[0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110], // 6
    &[0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000], // 7
    &[0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110], // 8
    &[0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100], // 9
];

fn draw_percentage_text(img: &mut RgbaImage, level: u8, color: Rgba<u8>) {
    let text = format!("{}", level.min(100));

    let char_count = text.len();
    let char_width = 5u32;
    let char_height = 7u32;
    let scale = if char_count == 3 { 2 } else { 3 };
    let spacing = 2u32;

    let total_width = (char_count as u32 * char_width * scale) + ((char_count as u32 - 1) * spacing);
    let start_x = (ICON_SIZE.saturating_sub(total_width)) / 2;
    let start_y = (ICON_SIZE.saturating_sub(char_height * scale)) / 2;

    let mut cur_x = start_x;
    for ch in text.chars() {
        if let Some(digit) = ch.to_digit(10) {
            let bitmap = DIGITS_5X7[digit as usize];
            for (row_idx, &row) in bitmap.iter().enumerate() {
                for col_idx in 0..5 {
                    if (row & (1 << (4 - col_idx))) != 0 {
                        for sy in 0..scale {
                            for sx in 0..scale {
                                let px = cur_x + (col_idx * scale) + sx;
                                let py = start_y + (row_idx as u32 * scale) + sy;
                                if px < ICON_SIZE && py < ICON_SIZE {
                                    img.put_pixel(px, py, color);
                                }
                            }
                        }
                    }
                }
            }
            cur_x += (char_width * scale) + spacing;
        }
    }
}

fn draw_error_icon(img: &mut RgbaImage) {
    let red = Rgba([255, 59, 48, 255]);
    for i in 6..26 {
        img.put_pixel(i, i, red);
        img.put_pixel(i + 1, i, red);
        img.put_pixel(31 - i, i, red);
        img.put_pixel(30 - i, i, red);
    }
}

fn make_background_transparent(img: &mut RgbaImage) {
    for pixel in img.pixels_mut() {
        if pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0 {
            pixel[3] = 0;
        }
    }
}
