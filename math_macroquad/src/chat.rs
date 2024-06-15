use macroquad::prelude::*;
use macroquad::text::draw_text;

struct TextBox {
    x: f32,
    y: f32,
    text: String,
}

#[macroquad::main("Canvas")]
async fn main() {
    let mut offset_x = 0.0;
    let mut offset_y = 0.0;
    let mut dragging = false;
    let mut last_mouse_x = 0.0;
    let mut last_mouse_y = 0.0;

    let mut text_boxes = Vec::new();
    let mut current_text = String::new();
    let mut typing = false;

    loop {
        clear_background(WHITE);

        // Handle panning
        if is_mouse_button_pressed(MouseButton::Middle) {
            dragging = true;
            last_mouse_x = mouse_position().0;
            last_mouse_y = mouse_position().1;
        }
        if is_mouse_button_released(MouseButton::Middle) {
            dragging = false;
        }
        if dragging {
            let (mouse_x, mouse_y) = mouse_position();
            offset_x += mouse_x - last_mouse_x;
            offset_y += mouse_y - last_mouse_y;
            last_mouse_x = mouse_x;
            last_mouse_y = mouse_y;
        }

        // Draw the grid
        for i in 0..50 {
            draw_line(
                i as f32 * 20.0 + offset_x % 20.0,
                0.0,
                i as f32 * 20.0 + offset_x % 20.0,
                screen_height(),
                1.0,
                LIGHTGRAY,
            );
            draw_line(
                0.0,
                i as f32 * 20.0 + offset_y % 20.0,
                screen_width(),
                i as f32 * 20.0 + offset_y % 20.0,
                1.0,
                LIGHTGRAY,
            );
        }

        // Handle text box creation and typing
        if is_mouse_button_pressed(MouseButton::Left) && !typing {
            let (mouse_x, mouse_y) = mouse_position();
            text_boxes.push(TextBox {
                x: mouse_x - offset_x,
                y: mouse_y - offset_y,
                text: String::new(),
            });
            typing = true;
        }

        if typing {
            let new_char = get_char_pressed();
            if new_char.is_some() {
                current_text.push(new_char.unwrap());
            }
            if is_key_pressed(KeyCode::Enter) {
                if let Some(text_box) = text_boxes.last_mut() {
                    text_box.text = current_text.clone();
                }
                current_text.clear();
                typing = false;
            }
        }

        // Draw text boxes
        for text_box in &text_boxes {
            draw_text(
                &text_box.text,
                text_box.x + offset_x,
                text_box.y + offset_y,
                20.0,
                BLACK,
            );
        }

        next_frame().await
    }
}
