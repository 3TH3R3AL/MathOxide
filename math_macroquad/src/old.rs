extern crate find_folder;
extern crate piston_window;
use std::fmt;
use std::ops::{Add, Sub};
use std::time::{SystemTime, UNIX_EPOCH};

struct Config {
    cursor_blink_rate: u128,
    backspace_interval_initial: f64,
    backspace_interval_ramp: f64,
}
const CONFIG: Config = Config {
    cursor_blink_rate: 500,
    backspace_interval_initial: 0.1,
    backspace_interval_ramp: 0.9,
};

#[derive(Debug, Clone)]
struct Coords {
    x: f64,
    y: f64,
}

impl Coords {
    fn distance(a: &Coords, b: &Coords) -> f64 {
        ((a.x - b.x).powf(2.0) + (a.y - b.y).powf(2.0)).sqrt()
    }
}

impl Add for Coords {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Sub for Coords {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl From<[f64; 2]> for Coords {
    fn from(arr: [f64; 2]) -> Self {
        Coords {
            x: arr[0],
            y: arr[1],
        }
    }
}

impl From<Coords> for [f64; 2] {
    fn from(coords: Coords) -> Self {
        [coords.x, coords.y]
    }
}
trait CanvasObject {
    fn is_empty(&self) -> bool {
        true
    }
    fn edit_text(&mut self, cursor: usize, text_input: String) {}
    fn backspace(&mut self, cursor: usize) {}
    fn edit_draw(
        &self,
        cursor: usize,
        fonts: &mut Fonts,
        c: &Context,
        g: &mut G2d,
        d: &mut GfxDevice,
        transform: [[f64; 3]; 2],
    ) {
    }

    fn draw(
        &self,
        fonts: &mut Fonts,
        c: &Context,
        g: &mut G2d,
        d: &mut GfxDevice,
        transform: [[f64; 3]; 2],
    ) {
    }
}

struct Comment {
    text: String,
    pos: Coords,
}
impl Comment {
    const FONT_SIZE: u32 = 33;
    const BLINK_INTERVAL: u128 = CONFIG.cursor_blink_rate;
}
impl CanvasObject for Comment {
    fn is_empty(&self) -> bool {
        self.text.len() == 0
    }
    fn edit_text(&mut self, cursor: usize, text_input: String) {
        self.text.insert_str(cursor, &text_input[..]);
    }
    fn backspace(&mut self, cursor: usize) {
        self.text.remove(cursor - 1);
    }
    fn draw(
        &self,
        fonts: &mut Fonts,
        c: &Context,
        g: &mut G2d,
        d: &mut GfxDevice,
        transform: [[f64; 3]; 2],
    ) {
        let glyphs = &mut fonts.comment_glyphs;
        let transform2 = transform.trans(self.pos.x, self.pos.y);
        let text_width = glyphs.width(Comment::FONT_SIZE, &self.text[..]).unwrap();
        rectangle(
            [0.0, 0.0, 0.0, 0.5],
            [
                0.0,
                -(Comment::FONT_SIZE as f64),
                text_width + (Comment::FONT_SIZE as f64),
                Comment::FONT_SIZE as f64 * 1.2,
            ],
            transform2,
            g,
        );
        text(
            [0.0, 0.0, 0.0, 1.0],
            Comment::FONT_SIZE,
            &self.text[..],
            glyphs,
            transform2,
            g,
        )
        .unwrap();
    }

    fn edit_draw(
        &self,
        cursor: usize,
        fonts: &mut Fonts,
        c: &Context,
        g: &mut G2d,
        d: &mut GfxDevice,
        transform: [[f64; 3]; 2],
    ) {
        let glyphs = &mut fonts.comment_glyphs;
        let transform2 = transform.trans(self.pos.x, self.pos.y);
        let mut text_to_draw = self.text.clone();
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let cursor_visible = (time / Comment::BLINK_INTERVAL) % 2 == 0;
        if cursor_visible {
            text_to_draw.insert(cursor, '|');
        }
        let text_width = glyphs.width(Comment::FONT_SIZE, &self.text[..]).unwrap();

        rectangle(
            [0.0, 0.0, 0.0, 0.5],
            [
                0.0,
                -(Comment::FONT_SIZE as f64),
                text_width + (Comment::FONT_SIZE as f64),
                Comment::FONT_SIZE as f64 * 1.2,
            ],
            transform2,
            g,
        );

        text(
            [0.0, 0.0, 0.0, 1.0],
            Comment::FONT_SIZE,
            &text_to_draw[..],
            glyphs,
            transform2,
            g,
        )
        .unwrap();
    }
}
use piston_window::{glyph_cache::rusttype::GlyphCache, modular_index::offset, *};
#[derive(Debug)]
enum CanvasState {
    Default,
    DraggingCanvas {
        start_offset: Coords,
        start_drag: Coords,
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
    offset: Coords,
    state: CanvasState,
    objects: Vec<Box<dyn CanvasObject>>,
}
impl Canvas {
    const GRID_SPACING: f64 = 50.0;
    fn new() -> Canvas {
        Canvas {
            offset: Coords { x: 0.0, y: 0.0 },
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
                if Coords::distance(&start_offset, &self.offset) < 2.0 {
                    self.state = CanvasState::Editing {
                        cursor: 0,
                        editing_object: Box::new(Comment {
                            text: String::from(""),
                            pos: mouse.cursor_pos.clone() - self.offset.clone(),
                        }),
                    };
                } else {
                    self.state = CanvasState::Default;
                }
            }
            _ => {}
        };
    }

    fn handle_typing(&mut self, text: String) {
        match self.state {
            CanvasState::Editing {
                ref mut editing_object,
                ref mut cursor,
            } => {
                let len = text.len();
                editing_object.edit_text(*cursor, text);
                *cursor += len;
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

    fn draw(&self, fonts: &mut Fonts, c: &Context, g: &mut G2d, d: &mut GfxDevice) {
        let size = c.get_view_size();
        let lines_x = (size[0] / Canvas::GRID_SPACING) as i32 + 3;
        let lines_y = (size[1] / Canvas::GRID_SPACING) as i32 + 3;

        // Draw vertical lines
        for i in -lines_x..lines_x {
            let x = i as f64 * Canvas::GRID_SPACING + self.offset.x % Canvas::GRID_SPACING;
            line(
                [0.5, 0.5, 0.5, 1.0],
                1.0,
                [x, 0.0, x, size[1]],
                c.transform,
                g,
            );
        }

        // Draw horizontal lines
        for j in -lines_y..lines_y {
            let y = j as f64 * Canvas::GRID_SPACING + self.offset.y % Canvas::GRID_SPACING;
            line(
                [0.5, 0.5, 0.5, 1.0],
                1.0,
                [0.0, y, size[0], y],
                c.transform,
                g,
            );
        }
        let transform = c.transform.trans(self.offset.x, self.offset.y);

        match &self.state {
            CanvasState::Editing {
                editing_object,
                cursor,
            } => {
                editing_object.edit_draw(*cursor, fonts, c, g, d, transform);
            }
            _ => {}
        };

        for object in &self.objects {
            object.draw(fonts, c, g, d, transform);
        }
    }
}

struct Mouse {
    cursor_pos: Coords,
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
    timer: f64,
    interval: f64,
}
struct Fonts {
    equation_glyphs: Glyphs,
    comment_glyphs: Glyphs,
}
struct State {
    canvas: Canvas,
    mouse: Mouse,
    fonts: Fonts,
    backspace: BackspaceState,
}
impl State {
    const BACKSPACE_INTERVAL_INITIAL: f64 = CONFIG.backspace_interval_initial;
    const BACKSPACE_INTERVAL_RAMP: f64 = CONFIG.backspace_interval_ramp;
    fn update(&mut self, event: &Event) {
        if let Some(Button::Mouse(MouseButton::Left)) = event.press_args() {
            self.mouse.is_down = true;
            self.canvas.handle_left_mouse_down(&self.mouse);
        }

        if let Some(Button::Mouse(MouseButton::Left)) = event.release_args() {
            self.mouse.is_down = false;
            self.canvas.handle_left_mouse_up(&self.mouse);
        }

        if let Some(pos) = event.mouse_cursor_args() {
            self.mouse.cursor_pos = pos.into();
            self.canvas.handle_mouse_move(&self.mouse);
        }
        if let Some(text) = event.text_args() {
            self.canvas.handle_typing(text);
        }

        if let Some(Button::Keyboard(Key::Backspace)) = event.press_args() {
            self.backspace.is_pressed = true;
            self.backspace.interval = State::BACKSPACE_INTERVAL_INITIAL;
            self.backspace.timer = 0.0;
            self.canvas.handle_backspace();
        }

        if let Some(Button::Keyboard(Key::Backspace)) = event.release_args() {
            self.backspace.is_pressed = false;
        }
        if let Some(args) = event.update_args() {
            if self.backspace.is_pressed
                && (args.dt + self.backspace.timer >= self.backspace.interval)
            {
                self.canvas.handle_backspace();
                self.backspace.timer = 0.0;
                self.backspace.interval *= State::BACKSPACE_INTERVAL_RAMP;
            } else {
                self.backspace.timer += args.dt;
            }
        }
    }
    fn draw(&mut self, c: &Context, g: &mut G2d, d: &mut GfxDevice) {
        self.canvas.draw(&mut self.fonts, c, g, d);

        self.fonts.equation_glyphs.factory.encoder.flush(d);
        self.fonts.comment_glyphs.factory.encoder.flush(d);
    }
}

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [640, 480])
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("fonts")
        .unwrap();
    let ref equation_font = assets.join("cmunso.ttf");
    let ref comment_font = assets.join("cmunbsr.ttf");
    let mut equation_glyphs = window.load_font(equation_font).unwrap();
    let mut comment_glyphs = window.load_font(comment_font).unwrap();

    let mut app_state = State {
        canvas: Canvas::new(),
        mouse: Mouse::new(),
        fonts: Fonts {
            equation_glyphs,
            comment_glyphs,
        },
        backspace: BackspaceState {
            is_pressed: false,
            interval: State::BACKSPACE_INTERVAL_INITIAL,
            timer: 0.0,
        },
    };
    while let Some(event) = window.next() {
        app_state.update(&event);
        window.draw_2d(&event, |c, g, d| {
            clear([1.0; 4], g); // Clear the screen with white color
            app_state.draw(&c, g, d);
        });
    }
}
