use macroquad::math::Vec2;
use macroquad::miniquad::RenderingBackend;
use macroquad::prelude::*;
use std::fmt;
use std::ops::{Add, Sub};
use std::time::{SystemTime, UNIX_EPOCH};
struct Config {
    cursor_blink_rate: u128,
    backspace_interval_initial: f32,
    backspace_interval_ramp: f32,
}
const CONFIG: Config = Config {
    cursor_blink_rate: 500,
    backspace_interval_initial: 0.1,
    backspace_interval_ramp: 0.9,
};

// impl Vec2 {
//     fn distance(a: &Vec2, b: &Vec2) -> f32 {
//         ((a.x - b.x).powf(2.0) + (a.y - b.y).powf(2.0)).sqrt()
//     }
// }
//
// impl Add for Vec2 {
//     type Output = Self;
//     fn add(self, other: Self) -> Self {
//         Self {
//             x: self.x + other.x,
//             y: self.y + other.y,
//         }
//     }
// }
// impl Sub for Vec2 {
//     type Output = Self;
//
//     fn sub(self, other: Self) -> Self {
//         Self {
//             x: self.x - other.x,
//             y: self.y - other.y,
//         }
//     }
// }
//
// impl From<[f32; 2]> for Vec2 {
//     fn from(arr: [f32; 2]) -> Self {
//         Vec2 {
//             x: arr[0],
//             y: arr[1],
//         }
//     }
// }
//
// impl From<(f32, f32)> for Vec2 {
//     fn from(arr: (f32, f32)) -> Self {
//         Vec2 { x: arr.0, y: arr.1 }
//     }
// }
// impl From<Vec2> for [f32; 2] {
//     fn from(coords: Vec2) -> Self {
//         [coords.x, coords.y]
//     }
// }
trait CanvasObject {
    fn is_empty(&self) -> bool {
        true
    }
    fn edit_text(&mut self, cursor: usize, text_input: char) {}
    fn backspace(&mut self, cursor: usize) {}
    fn edit_draw(&self, cursor: usize, fonts: &mut Fonts) {}

    fn draw(&self, fonts: &mut Fonts) {}
}

struct Comment {
    text: String,
    pos: Vec2,
}
impl Comment {
    const FONT_SIZE: u16 = 33;
    const BLINK_INTERVAL: u128 = CONFIG.cursor_blink_rate;
}
impl CanvasObject for Comment {
    fn is_empty(&self) -> bool {
        self.text.len() == 0
    }
    fn edit_text(&mut self, cursor: usize, text_input: char) {
        self.text.insert(cursor, text_input)
    }
    fn backspace(&mut self, cursor: usize) {
        self.text.remove(cursor - 1);
    }
    fn draw(&self, fonts: &mut Fonts) {
        let font = &mut fonts.comments;

        let text_dimensions = measure_text(&self.text[..], Some(font), Comment::FONT_SIZE, 1.0);

        draw_rectangle(
            self.pos.x,
            self.pos.y - (Comment::FONT_SIZE as f32),
            text_dimensions.width + (Comment::FONT_SIZE) as f32,
            Comment::FONT_SIZE as f32 * 1.2,
            color_u8!(0, 0, 0, 128),
        );

        draw_text_ex(
            &self.text[..],
            self.pos.x,
            self.pos.y,
            TextParams {
                font: Some(font),
                font_size: Comment::FONT_SIZE,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                rotation: 0.0,
                color: color_u8!(0, 0, 0, 1),
            },
        );
    }

    fn edit_draw(&self, cursor: usize, fonts: &mut Fonts) {
        let font = &mut fonts.comments;
        let mut text_to_draw = self.text.clone();
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let cursor_visible = (time / Comment::BLINK_INTERVAL) % 2 == 0;
        if cursor_visible {
            text_to_draw.insert(cursor, '|');
        }
        let text_dimensions = measure_text(&text_to_draw[..], Some(font), Comment::FONT_SIZE, 1.0);

        draw_rectangle_ex(
            0.0,
            -(Comment::FONT_SIZE as f32),
            text_dimensions.width + (Comment::FONT_SIZE) as f32,
            Comment::FONT_SIZE as f32 * 1.2,
            DrawRectangleParams {
                offset: self.pos,
                rotation: 0.0,
                color: color_u8!(0, 0, 0, 128),
            },
        );

        draw_text_ex(
            &text_to_draw[..],
            self.pos.x,
            self.pos.y,
            TextParams {
                font: Some(font),
                font_size: Comment::FONT_SIZE,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                rotation: 0.0,
                color: color_u8!(0, 0, 0, 1),
            },
        );
    }
}
#[derive(Debug)]
enum CanvasState {
    Default,
    DraggingCanvas {
        start_offset: Vec2,
        start_drag: Vec2,
    },
    Editing {
        cursor: usize,
        editing_object: Box<dyn CanvasObject>,
    },
    DraggingObject,
}

impl fmt::Debug for Box<dyn CanvasObject> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Box<dyn CanvasObject>")
    }
}
struct Canvas {
    offset: Vec2,
    state: CanvasState,
    objects: Vec<Box<dyn CanvasObject>>,
}
impl Canvas {
    const GRID_SPACING: f32 = 50.0;
    const LINE_COLOR: Color = color_u8!(0, 0, 0, 200);
    fn new() -> Canvas {
        Canvas {
            offset: Vec2 { x: 0.0, y: 0.0 },
            state: CanvasState::Default,
            objects: Vec::new(),
        }
    }

    fn handle_left_mouse_down(&mut self, mouse: &Mouse) {
        let mut temp_state = std::mem::replace(&mut self.state, CanvasState::Default);
        match temp_state {
            CanvasState::Editing { editing_object, .. } => {
                if !editing_object.is_empty() {
                    self.objects.push(editing_object);
                }
            }
            _ => {}
        }
        self.state = CanvasState::DraggingCanvas {
            start_drag: mouse.cursor_pos.clone(),
            start_offset: self.offset.clone(),
        };
    }

    fn handle_mouse_move(&mut self, mouse: &Mouse) {
        match &self.state {
            CanvasState::DraggingCanvas {
                start_offset,
                start_drag,
            } => {
                self.offset.x = start_offset.x - start_drag.x + mouse.cursor_pos.x;
                self.offset.y = start_offset.y - start_drag.y + mouse.cursor_pos.y;
                println!("{:?}", self.offset);
            }
            _ => {}
        }
    }

    fn handle_left_mouse_up(&mut self, mouse: &Mouse) {
        match &self.state {
            CanvasState::DraggingCanvas {
                start_offset,
                start_drag,
            } => {
                if Vec2::distance(*start_offset, self.offset.clone()) < 2.0 {
                    self.state = CanvasState::Editing {
                        cursor: 0,
                        editing_object: Box::new(Comment {
                            text: String::from(""),
                            pos: mouse.cursor_pos.clone() - self.offset.clone(),
                        }),
                    };
                    println!("{:?}", mouse.cursor_pos.clone() - self.offset.clone());
                } else {
                    self.state = CanvasState::Default;
                }
            }
            _ => {}
        };
    }

    fn handle_typing(&mut self, text: char) {
        match self.state {
            CanvasState::Editing {
                ref mut editing_object,
                ref mut cursor,
            } => {
                editing_object.edit_text(*cursor, text);
                *cursor += 1;
            }
            _ => {}
        }
    }

    fn handle_backspace(&mut self) {
        match self.state {
            CanvasState::Editing {
                ref mut editing_object,
                ref mut cursor,
            } => {
                if *cursor == 0 as usize {
                    return;
                }
                editing_object.backspace(*cursor);
                *cursor -= 1;
            }
            _ => {}
        }
    }

    fn draw(&self, fonts: &mut Fonts) {
        let lines_x = (screen_width() / Canvas::GRID_SPACING) as i32 + 3;
        let lines_y = (screen_height() / Canvas::GRID_SPACING) as i32 + 3;

        // Draw vertical lines
        for i in -lines_x..lines_x {
            let x = i as f32 * Canvas::GRID_SPACING + self.offset.x % Canvas::GRID_SPACING;
            draw_line(x, 0.0, x, screen_height(), 1.0, Canvas::LINE_COLOR);
        }

        // Draw horizontal lines
        for j in -lines_y..lines_y {
            let y = j as f32 * Canvas::GRID_SPACING + self.offset.y % Canvas::GRID_SPACING;
            draw_line(0.0, y, screen_width(), y, 1.0, Canvas::LINE_COLOR);
        }

        match &self.state {
            CanvasState::Editing {
                editing_object,
                cursor,
            } => {
                editing_object.edit_draw(*cursor, fonts);
            }
            _ => {}
        };

        for object in &self.objects {
            object.draw(fonts);
        }
    }
}

struct Mouse {
    cursor_pos: Vec2,
    is_down: bool,
}
impl Mouse {
    fn new() -> Mouse {
        Mouse {
            cursor_pos: [0.0, 0.0].into(),
            is_down: false,
        }
    }
}
struct BackspaceState {
    is_pressed: bool,
    timer: f32,
    interval: f32,
}
struct Fonts {
    equations: Font,
    comments: Font,
}
struct State {
    canvas: Canvas,
    mouse: Mouse,
    fonts: Fonts,
    backspace: BackspaceState,
}
impl State {
    const BACKSPACE_INTERVAL_INITIAL: f32 = CONFIG.backspace_interval_initial;
    const BACKSPACE_INTERVAL_RAMP: f32 = CONFIG.backspace_interval_ramp;
    fn update(&mut self) {
        if is_mouse_button_pressed(MouseButton::Left) {
            self.mouse.is_down = true;
            self.canvas.handle_left_mouse_down(&self.mouse);
        }

        if is_mouse_button_released(MouseButton::Left) {
            self.mouse.is_down = false;
            self.canvas.handle_left_mouse_up(&self.mouse);
        }

        if is_mouse_button_down(MouseButton::Left) {
            self.mouse.cursor_pos = Vec2::from(mouse_position());
            self.canvas.handle_mouse_move(&self.mouse);
        }
        if let Some(text) = get_char_pressed() {
            self.canvas.handle_typing(text);
        }

        if is_key_pressed(KeyCode::Backspace) {
            self.backspace.is_pressed = true;
            self.backspace.interval = State::BACKSPACE_INTERVAL_INITIAL;
            self.backspace.timer = 0.0;
            self.canvas.handle_backspace();
        }

        if is_key_released(KeyCode::Backspace) {
            self.backspace.is_pressed = false;
        }
        if self.backspace.is_pressed
            && (get_frame_time() + self.backspace.timer >= self.backspace.interval)
        {
            self.canvas.handle_backspace();
            self.backspace.timer = 0.0;
            self.backspace.interval *= State::BACKSPACE_INTERVAL_RAMP;
        } else {
            self.backspace.timer += get_frame_time();
        }
    }
    fn draw(&mut self) {
        self.canvas.draw(&mut self.fonts);
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let equation_font = load_ttf_font_from_bytes(include_bytes!("../assets/cmunso.ttf")).unwrap();
    let comment_font = load_ttf_font_from_bytes(include_bytes!("../assets/cmunbsr.ttf")).unwrap();

    let mut app_state = State {
        canvas: Canvas::new(),
        mouse: Mouse::new(),
        fonts: Fonts {
            equations: equation_font,
            comments: comment_font,
        },
        backspace: BackspaceState {
            is_pressed: false,
            interval: State::BACKSPACE_INTERVAL_INITIAL,
            timer: 0.0,
        },
    };
    loop {
        clear_background(WHITE);
        app_state.update();
        app_state.draw();
        next_frame().await;
    }
}
