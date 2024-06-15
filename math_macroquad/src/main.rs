#![no_std]
use macroquad::math::Vec2;
use macroquad::miniquad::window::{quit, set_window_position, show_keyboard};
use macroquad::prelude::*;
//use std::ops::{Add, Sub};
use instant::Instant;
extern crate alloc;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
extern crate simplelog;
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
struct Config {
    cursor_blink_rate: u128,
    backspace_interval_initial: f32,
    backspace_interval_ramp: f32,
    click_distance: f32,
}
const CONFIG: Config = Config {
    cursor_blink_rate: 500,
    backspace_interval_initial: 0.1,
    backspace_interval_ramp: 0.9,
    click_distance: 2.0,
};
fn draw_rounded_rectangle(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    line_thickness: f32,
    line_color: Color,
    fill_color: Color,
) {
    draw_line(x + radius, y, x + w - radius, y, line_thickness, line_color);
    draw_line(
        x + radius,
        y + h,
        x + w - radius,
        y + h,
        line_thickness,
        line_color,
    );
    draw_line(x, y + radius, x, y + h - radius, line_thickness, line_color);
    draw_line(
        x + w,
        y + radius,
        x + w,
        y + h - radius,
        line_thickness,
        line_color,
    );

    draw_circle_lines(x + radius, y + radius, radius, line_thickness, line_color);

    draw_circle_lines(
        x + w - radius,
        y + radius,
        radius,
        line_thickness,
        line_color,
    );

    draw_circle_lines(
        x + radius,
        y + h - radius,
        radius,
        line_thickness,
        line_color,
    );

    draw_circle_lines(
        x + w - radius,
        y + h - radius,
        radius,
        line_thickness,
        line_color,
    );
    draw_rectangle(x + radius, y, w - radius * 2., h, fill_color);
    draw_rectangle(x, y + radius, w, h - radius * 2., fill_color);

    draw_circle(x + radius, y + radius, radius, fill_color);

    draw_circle(x + w - radius, y + radius, radius, fill_color);

    draw_circle(x + radius, y + h - radius, radius, fill_color);

    draw_circle(x + w - radius, y + h - radius, radius, fill_color);
}
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
    fn edit_text(&mut self, cursor: &mut usize, text_input: char) {}
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
}
impl CanvasObject for Comment {
    fn is_empty(&self) -> bool {
        self.text.len() == 0
    }
    fn edit_text(&mut self, cursor: &mut usize, text_input: char) {
        if text_input.is_ascii_graphic() || text_input.is_ascii_whitespace() {
            self.text.insert(*cursor, text_input);
            *cursor += 1;
            info!("{}", self.text);
        }
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
            text_dimensions.width,
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
                color: color_u8!(0, 0, 0, 255),
                ..Default::default()
            },
        );
    }

    fn edit_draw(&self, cursor: usize, fonts: &mut Fonts) {
        let font = &mut fonts.comments;
        let mut text_to_draw = self.text.clone();
        let time = instant::now();
        let cursor_visible = (time as u128 / CONFIG.cursor_blink_rate) % 2 == 0;
        if cursor_visible {
            //info!("cursor: {}-{:?}", cursor, text_to_draw);
            text_to_draw.insert(cursor, '|');
        }
        let text_dimensions = measure_text(&self.text[..], Some(font), Comment::FONT_SIZE, 1.0);

        draw_rectangle(
            self.pos.x,
            self.pos.y - (Comment::FONT_SIZE as f32),
            text_dimensions.width + (Comment::FONT_SIZE) as f32,
            Comment::FONT_SIZE as f32 * 1.2,
            color_u8!(0, 0, 0, 128),
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
                color: color_u8!(0, 0, 0, 255),
            },
        );
    }
}
/// A Mathematical Equation on the Canvas
struct Equation {
    text: String,
    pos: Vec2,
}

impl Equation {
    const FONT_SIZE: u16 = 33;
}
impl CanvasObject for Equation {
    fn is_empty(&self) -> bool {
        self.text.len() == 0
    }
    fn edit_text(&mut self, cursor: &mut usize, text_input: char) {
        if text_input.is_ascii_graphic() || text_input.is_ascii_whitespace() {
            self.text.insert(*cursor, text_input);
            *cursor += 1;
            info!("{}", self.text);
        }
    }
    fn backspace(&mut self, cursor: usize) {
        self.text.remove(cursor - 1);
    }
    fn draw(&self, fonts: &mut Fonts) {
        let font = &mut fonts.equations;

        let text_dimensions = measure_text(&self.text[..], Some(font), Comment::FONT_SIZE, 1.0);

        draw_rectangle(
            self.pos.x,
            self.pos.y - (Comment::FONT_SIZE as f32),
            text_dimensions.width,
            Comment::FONT_SIZE as f32 * 1.2,
            color_u8!(0, 0, 0, 10),
        );

        draw_text_ex(
            &self.text[..],
            self.pos.x,
            self.pos.y,
            TextParams {
                font: Some(font),
                font_size: Comment::FONT_SIZE,
                color: color_u8!(0, 0, 0, 255),
                ..Default::default()
            },
        );
    }

    fn edit_draw(&self, cursor: usize, fonts: &mut Fonts) {
        let font = &mut fonts.equations;
        let mut text_to_draw = self.text.clone();
        let time = instant::now();
        let cursor_visible = (time as u128 / CONFIG.cursor_blink_rate) % 2 == 0;
        if cursor_visible {
            //info!("cursor: {}-{:?}", cursor, text_to_draw);
            text_to_draw.insert(cursor, '|');
        }
        let text_dimensions = measure_text(&self.text[..], Some(font), Comment::FONT_SIZE, 1.0);

        draw_rectangle(
            self.pos.x,
            self.pos.y - (Comment::FONT_SIZE as f32),
            text_dimensions.width + (Comment::FONT_SIZE) as f32,
            Comment::FONT_SIZE as f32 * 1.2,
            color_u8!(0, 0, 0, 10),
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
                color: color_u8!(0, 0, 0, 255),
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
    target: Vec2,
    state: CanvasState,
    objects: Vec<Box<dyn CanvasObject>>,
    camera: Camera2D,
}
impl Canvas {
    const GRID_SPACING: f32 = 50.0;
    const LINE_COLOR: Color = color_u8!(0, 0, 0, 200);
    fn new() -> Canvas {
        Canvas {
            target: Vec2 { x: 0.0, y: 0.0 },
            state: CanvasState::Default,
            camera: Camera2D {
                rotation: 0.0,
                // I HAVE NO IDEA WHY THIS NEEDS TO BE THIS WAY BUT IT WORKS NOW AND I'M NOT
                // SPENDING ANY MORE TIME ON IT
                zoom: Vec2::new(1. / screen_width() * 2., 1. / screen_height() * 2.),
                target: Vec2::new(0., 0.),
                offset: Vec2::new(0.0, 0.0),
                render_target: None,
                viewport: None,
            },

            objects: Vec::new(),
        }
    }

    fn insert_object_if_editing(&mut self) {
        let temp_state = core::mem::replace(&mut self.state, CanvasState::Default);
        match temp_state {
            CanvasState::Editing { editing_object, .. } => {
                if !editing_object.is_empty() {
                    self.objects.push(editing_object);
                }
            }
            _ => {}
        }

        show_keyboard(false);
    }
    fn start_drag(&mut self) {
        self.state = CanvasState::DraggingCanvas {
            start_drag: Vec2::from(mouse_position()),
            start_offset: self.camera.target.clone(),
        };
    }

    fn handle_mouse_move(&mut self) {
        match &self.state {
            CanvasState::DraggingCanvas {
                start_offset,
                start_drag,
            } => {
                self.camera.target = *start_offset + *start_drag - Vec2::from(mouse_position());
                //info!("{:?}", self.camera.target);
            }
            _ => {}
        }
    }
    fn is_click(&self) -> bool {
        match &self.state {
            CanvasState::DraggingCanvas {
                start_offset,
                start_drag,
            } => Vec2::distance(*start_offset, self.camera.target.clone()) < 2.0,
            _ => false,
        }
    }
    fn start_edit(&mut self, editing_object: Box<dyn CanvasObject>) {
        self.state = CanvasState::Editing {
            cursor: 0,
            editing_object,
        };
        show_keyboard(true);
        info!(
            "Inserted at {:?}",
            Vec2::from(mouse_position()) + self.camera.target.clone()
        );
    }

    fn handle_typing(&mut self, text: char) {
        match self.state {
            CanvasState::Editing {
                ref mut editing_object,
                ref mut cursor,
            } => {
                editing_object.edit_text(cursor, text);
            }
            _ => {}
        }
    }

    fn handle_backspace(&mut self, tool: &Tool) {
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

    fn draw(&mut self, fonts: &mut Fonts) {
        let lines_x = (screen_width() / Canvas::GRID_SPACING) as i32 + 3;
        let lines_y = (screen_height() / Canvas::GRID_SPACING) as i32 + 3;

        // Draw vertical lines
        for i in -lines_x..lines_x {
            let x = i as f32 * Canvas::GRID_SPACING - self.camera.target.x % Canvas::GRID_SPACING;
            draw_line(x, 0.0, x, screen_height(), 1.0, Canvas::LINE_COLOR);
        }

        // Draw horizontal lines
        for j in -lines_y..lines_y {
            let y = j as f32 * Canvas::GRID_SPACING - self.camera.target.y % Canvas::GRID_SPACING;
            draw_line(0.0, y, screen_width(), y, 1.0, Canvas::LINE_COLOR);
        }

        self.camera.zoom = Vec2::new(1. / screen_width() * 2., 1. / screen_height() * 2.);
        set_camera(&self.camera);
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
        set_default_camera();
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
enum Tool {
    Equation,
    Comment,
}
struct ToolButton {
    position: Vec2,
    symbol: Texture2D,
}

impl ToolButton {
    const DIMENSIONS: Vec2 = Vec2::new(60., 60.);
    fn is_mouse_over(&self) -> bool {
        let mouse = mouse_position();
        mouse.0 > self.position.x
            && mouse.0 < self.position.x + Self::DIMENSIONS.x
            && mouse.1 > self.position.y
            && mouse.1 < self.position.y + Self::DIMENSIONS.y
    }
    fn draw(&self) {
        let fill_color = if self.is_mouse_over() {
            color_u8!(100, 100, 100, 255)
        } else {
            color_u8!(200, 200, 200, 255)
        };
        draw_rounded_rectangle(
            self.position.x,
            self.position.y,
            Self::DIMENSIONS.x,
            Self::DIMENSIONS.y,
            20.,
            2.,
            color_u8!(0, 0, 0, 255),
            fill_color,
        );
        draw_texture_ex(
            &self.symbol,
            self.position.x,
            self.position.y,
            color_u8!(255, 255, 255, 255),
            DrawTextureParams {
                dest_size: Some(ToolButton::DIMENSIONS),
                ..Default::default()
            },
        );
    }
}
struct State {
    canvas: Canvas,
    tool: Tool,
    tool_buttons: Vec<ToolButton>,
    fonts: Fonts,
    backspace: BackspaceState,
}
impl State {
    const BACKSPACE_INTERVAL_INITIAL: f32 = CONFIG.backspace_interval_initial;
    const BACKSPACE_INTERVAL_RAMP: f32 = CONFIG.backspace_interval_ramp;
    fn update(&mut self) {
        // TODO: Clean this up with nested match statements
        match self.tool {
            Tool::Comment | Tool::Equation => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    self.canvas.start_drag();
                    self.canvas.insert_object_if_editing();
                }

                if is_mouse_button_released(MouseButton::Left) {
                    if self.canvas.is_click() {
                        let tool_index =
                            self.tool_buttons
                                .iter()
                                .enumerate()
                                .fold(None, |prev, (i, button)| {
                                    if button.is_mouse_over() {
                                        Some(i)
                                    } else {
                                        prev
                                    }
                                });
                        match tool_index {
                            Some(index) => {
                                info!("ToolIndex: {}", index);
                                self.canvas.state = CanvasState::Default;
                            }
                            None => match self.tool {
                                Tool::Comment => {
                                    self.canvas.start_edit(Box::new(Comment {
                                        text: String::from(""),
                                        pos: Vec2::from(mouse_position())
                                            + self.canvas.camera.target.clone()
                                            - Vec2::new(screen_width() / 2., screen_height() / 2.),
                                    }));
                                }
                                Tool::Equation => {
                                    self.canvas.start_edit(Box::new(Equation {
                                        text: String::from(""),
                                        pos: Vec2::from(mouse_position())
                                            + self.canvas.camera.target.clone()
                                            - Vec2::new(screen_width() / 2., screen_height() / 2.),
                                    }));
                                }
                            },
                        }
                    } else {
                        self.canvas.state = CanvasState::Default;
                    }
                }
                if is_mouse_button_down(MouseButton::Left) {
                    self.canvas.handle_mouse_move();
                }

                if let Some(text) = get_char_pressed() {
                    self.canvas.handle_typing(text);
                }
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            quit();
        }
        if is_mouse_button_down(MouseButton::Left) {
            self.canvas.handle_mouse_move();
        }

        if is_key_pressed(KeyCode::Backspace) {
            self.backspace.is_pressed = true;
            self.backspace.interval = State::BACKSPACE_INTERVAL_INITIAL;
            self.backspace.timer = 0.0;
            self.canvas.handle_backspace(&self.tool);
        }

        if is_key_released(KeyCode::Backspace) {
            self.backspace.is_pressed = false;
        }

        if self.backspace.is_pressed
            && (get_frame_time() + self.backspace.timer >= self.backspace.interval)
        {
            self.canvas.handle_backspace(&self.tool);
            self.backspace.timer = 0.0;
            self.backspace.interval *= State::BACKSPACE_INTERVAL_RAMP;
        } else {
            self.backspace.timer += get_frame_time();
        }
    }
    fn draw(&mut self) {
        self.canvas.draw(&mut self.fonts);

        self.tool_buttons.iter().for_each(|button| button.draw());
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default());
    //    #[cfg(target_arch = "wasm32")]

    let equation_font = load_ttf_font_from_bytes(include_bytes!("../assets/cmunso.ttf")).unwrap();
    let comment_font = load_ttf_font_from_bytes(include_bytes!("../assets/cmunbsr.ttf")).unwrap();

    let mut app_state = State {
        canvas: Canvas::new(),
        tool: Tool::Comment,
        tool_buttons: Vec::new(),
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
    app_state.tool_buttons.push(ToolButton {
        position: Vec2::new(50., 50.),
        symbol: load_texture("assets/sigma.png").await.unwrap(),
    });
    //set_window_position(500, 0);
    set_fullscreen(true);
    loop {
        clear_background(WHITE);
        app_state.update();
        app_state.draw();
        next_frame().await;
    }
}
