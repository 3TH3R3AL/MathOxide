#![no_std]
use macroquad::math::Vec2;
use macroquad::miniquad::window::{quit, show_keyboard};
use macroquad::prelude::*;
//use std::ops::{Add, Sub};
extern crate alloc;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{self, Display};
use core::num;
use core::str::FromStr;
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
            //info!("{}", self.text);
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
#[derive(Clone)]
enum Term {
    Empty,
    Cursor,
    Numeral(u128, Option<u32>), // DIY Floating point for decimal digits
    Negative(usize),
    Variable(String),
    Multiplication(Vec<usize>),
    Addition(Vec<usize>),
    Division(usize, usize),
    Parentheses(usize),
    Exponentiation(usize, usize),
}
#[derive(Clone)]
struct TermNode {
    idx: usize,
    term: Term,
    parent: Option<usize>,
}
#[derive(Debug, PartialEq, Eq)]
struct ParseTermError;
impl Term {
    fn is_operator(&self) -> bool {
        match self {
            Self::Addition(..)
            | Self::Negative(..)
            | Self::Multiplication(..)
            | Self::Division { .. }
            | Self::Exponentiation(..) => true,
            _ => false,
        }
    }
}
struct EquationTree {
    arena: Vec<TermNode>,
}
impl EquationTree {
    fn new() -> Self {
        let init = Vec::from([TermNode {
            idx: 0,
            term: Term::Empty,
            parent: None,
        }]);
        Self { arena: init }
    }
    fn get(&self, idx: usize) -> TermNode {
        self.arena[idx].clone()
    }

    fn set(&mut self, idx: usize, node: TermNode) {
        debug_assert!(self.arena.len() > idx);
        debug_assert!(node.idx == idx);
        self.arena[idx] = node;
    }
    fn set_parent(&mut self, idx: usize, parent: usize) {
        debug_assert!(self.arena.len() > idx);
        self.arena[idx].parent = Some(parent);
    }
    fn push(&mut self, mut node: TermNode) -> usize {
        node.idx = self.arena.len();
        self.arena.push(node);
        self.arena.len() - 1
    }
    fn push_term(&mut self, term: Term, parent: usize) -> usize {
        let node = TermNode {
            idx: self.arena.len(),
            parent: Some(parent),
            term,
        };
        self.arena.push(node);
        self.arena.len() - 1
    }

    fn form_mult(&mut self, original_term: TermNode, new_term: Term) -> Term {
        let new_o_id = self.push(TermNode {
            parent: Some(original_term.idx),
            ..original_term
        });
        let new_term_id = self.push_term(new_term, original_term.idx);
        Term::Multiplication(Vec::from([new_o_id, new_term_id]))
    }
    fn replace_child(&mut self, child: TermNode, new_term: Term) -> usize {
        let new_child = self.push_term(new_term, child.parent.unwrap());
        let parent = self.get(child.parent.unwrap());
        match parent.term {
            Term::Parentheses(ref child_idx) | Term::Negative(ref child_idx) => {
                debug_assert!(*child_idx == child.idx);
                *child_idx = new_child;
            }
            Term::Multiplication(ref children) | Term::Addition(ref children) => {
                let index = children.iter().position(|&x| x == child.idx).unwrap();
                children[index] = new_child;
            }
            Term::Division(ref child1, ref child2)
            | Term::Exponentiation(ref child1, ref child2) => {
                if *child1 == child.idx {
                    *child1 = new_child;
                }
                if *child2 == child.idx {
                    *child2 = new_child;
                }
            }
            _ => {
                panic!("ERROR: TRIED TO REPLACE CHILD OF BAD TERM");
            }
        };
        new_child
    }

    /// Returns new cursor location
    fn append_mult(&mut self, original_term: &TermNode, new_term: Term) -> usize {
        let parent = self.get(original_term.parent.unwrap_or(0)); // Might be a bug idk
        match parent.term {
            Term::Multiplication(mut children) => {
                let to_append_location = self.push_term(new_term, parent.idx);
                children.push(to_append_location);
                to_append_location
            }
            _ => {
                let new_mult_location = original_term.idx;
                self.set(
                    original_term.idx,
                    TermNode {
                        term: Term::Multiplication(Vec::new()),
                        ..original_term
                    },
                );
                let new_o_location = self.push(original_term);
                self.set_parent(new_o_location, new_mult_location);
                let new_term_location = self.push_term(new_term, new_mult_location);
                new_term_location
            }
        }
    }
}
impl FromStr for EquationTree {
    type Err = ParseTermError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut root = EquationTree::new();
        let mut empty_term = Some(0);
        let mut adding_term: Option<usize> = None;
        let mut chars = s.chars();
        for char in chars {
            debug_assert!(empty_term.is_none() || adding_term.is_none());
            if let Some(cursor) = empty_term {
                let current_term = root.get(cursor);
                debug_assert!(if let Term::Empty = current_term.term {
                    true
                } else {
                    false
                });
                let new_node = TermNode {
                    term: match char {
                        '0'..='9' => Term::Numeral(char.to_digit(10).unwrap() as u128, None),
                        'a'..='z' | 'A'..='Z' => Term::Variable(String::from(char)),

                        '(' => Term::Parentheses(root.push_term(Term::Empty, cursor)),
                        '-' => Term::Negative(root.push_term(Term::Empty, cursor)),
                        '\u{FF5C}' => Term::Cursor,
                        _ => {
                            return Err(ParseTermError);
                        }
                    },
                    ..current_term
                };
                root.set(cursor, new_node);
                (empty_term, adding_term) = match char {
                    '\u{FF5C}' | '0'..='9' | 'a'..='z' | 'A'..='Z' => (None, Some(cursor)),
                    '(' | '-' => {
                        if let Term::Parentheses(empty) = current_term.term {
                            (Some(empty), None)
                        } else if let Term::Negative(empty) = current_term.term {
                            (Some(empty), None)
                        } else {
                            return Err(ParseTermError);
                        }
                    }
                    _ => {
                        return Err(ParseTermError);
                    }
                };
                continue;
            }
            if let Some(mut cursor) = adding_term {
                let current_term = root.get(cursor);
                let new_node = TermNode {
                    term: match (&current_term.term, char) {
                        (Term::Numeral(current, None), '0'..='9') => {
                            Term::Numeral(current * 10 + char.to_digit(10).unwrap() as u128, None)
                        }
                        (Term::Numeral(current, Some(exp)), '0'..='9') => Term::Numeral(
                            current * 10 + char.to_digit(10).unwrap() as u128,
                            Some(exp + 1),
                        ),
                        (Term::Numeral(..) | Term::Variable(..) | Term::Parentheses(..), '^') => {
                            // info!("expo attempted")
                            Term::Exponentiation(
                                root.push_term(current_term.term.clone(), cursor),
                                root.push_term(Term::Empty, cursor),
                            )
                        }
                        (Term::Variable(var_name), '_' | 'a'..='z' | 'A'..='Z') => {
                            if var_name.chars().count() > 1 || char == '_' {
                                Term::Variable(format!("{}{}", var_name, char))
                            } else {
                                root.append_mult(current_term, Term::Variable(char.to_string()))
                                Term::Variable(char.to_string())
                            }
                        }
                        // ( => {terms.append(other);},

                        // '(' => Term::Parentheses(Box::from(Term::Empty)),
                        // '-' => Term::Negative(Box::from(Term::Empty)),
                        // '\u{FF5C}' => Term::Cursor,
                        _ => {
                            return Err(ParseTermError);
                        }
                    },
                    ..current_term
                };
                (empty_term, adding_term) = match char {
                    '\u{FF5C}' | '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' => (None, Some(cursor)),
                    '(' | '-' | '^' => match new_node.term {
                        Term::Parentheses(empty)
                        | Term::Negative(empty)
                        | Term::Exponentiation(_, empty) => (Some(empty), None),
                        _ => {
                            return Err(ParseTermError);
                        }
                    },

                    _ => {
                        return Err(ParseTermError);
                    }
                };

                root.set(cursor, new_node);
            }
        }
        Ok(root)
    }
}

impl TermNode {
    fn to_string(&self, tree: &EquationTree) -> String {
        use Term::*;
        match &self.term {
            Empty => "(Empty)".to_string(),
            Cursor => "(Cursor)".to_string(),
            Numeral(num, exp) => {
                format!(
                    "Num[{}]",
                    (*num as f64 / (u128::pow(10, exp.unwrap_or(0)) as f64))
                )
            }

            Variable(name) => format!("Var[{}]", name.clone()),
            Parentheses(term) => format!("({})", tree.get(*term).to_string(tree)),
            Negative(term) => format!("Negative({})", tree.get(*term).to_string(tree)),
            Multiplication(terms) => format!(
                "Multiplication({})",
                terms.iter().fold(String::new(), |acc, &term| format!(
                    "{}{},",
                    acc,
                    tree.get(term).to_string(tree)
                ))
            ),
            Addition(terms) => format!(
                "Addition({})",
                terms.iter().fold(String::new(), |acc, &term| format!(
                    "{}{},",
                    acc,
                    tree.get(term).to_string(tree)
                ))
            ),
            Division(term, term2) => format!(
                "Division({}/{})",
                tree.get(*term).to_string(tree),
                tree.get(*term2).to_string(tree)
            ),
            Exponentiation(term, term2) => format!(
                "Exponentiation({}^{})",
                tree.get(*term).to_string(tree),
                tree.get(*term2).to_string(tree)
            ),
        }
    }
}
impl Display for EquationTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EquationTree:\n{}", self.get(0).to_string(self))
    }
}
/// A Mathematical Equation on the Canvas
struct Equation {
    text: String,
    pos: Vec2,
}

impl Equation {
    const FONT_SIZE: u16 = 33;
    fn replace_symbols(&self) -> String {
        Equation::replace_symbols_str(&self.text)
    }

    fn replace_symbols_str(text: &String) -> String {
        text.replace("pi", "\u{03C0}")
            .replace("theta", "\u{03B8}")
            .replace("*", "\u{00B7}")
    }
}
impl CanvasObject for Equation {
    fn is_empty(&self) -> bool {
        self.text.len() == 0
    }
    fn edit_text(&mut self, cursor: &mut usize, text_input: char) {
        if text_input.is_ascii_graphic() || text_input.is_ascii_whitespace() {
            self.text.insert(*cursor, text_input);
            *cursor += 1;
            // info!("{}", self.text);
        }
    }
    fn backspace(&mut self, cursor: usize) {
        self.text.remove(cursor - 1);
    }
    fn draw(&self, fonts: &mut Fonts) {
        let font = &mut fonts.equations;

        let text_to_draw = self.replace_symbols();
        let text_dimensions = measure_text(&text_to_draw[..], Some(font), Comment::FONT_SIZE, 1.0);

        draw_rectangle(
            self.pos.x,
            self.pos.y - (Comment::FONT_SIZE as f32),
            text_dimensions.width,
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
                color: color_u8!(0, 0, 0, 255),
                ..Default::default()
            },
        );
    }

    fn edit_draw(&self, cursor: usize, fonts: &mut Fonts) {
        let font = &mut fonts.equations;
        let mut text_to_draw = self.text.clone();

        let equation_tree = EquationTree::from_str(&self.text[..]);
        match equation_tree {
            Ok(tree) => info!("{}", tree),
            Err(_) => info!("Parsing Error :("),
        }
        let time = instant::now();
        let cursor_visible = (time as u128 / CONFIG.cursor_blink_rate) % 2 == 0;
        if cursor_visible {
            //info!("cursor: {}-{:?}", cursor, text_to_draw);
            text_to_draw.insert(cursor, '\u{FF5C}');
        }

        let text_to_draw = Equation::replace_symbols_str(&self.text);
        let text_dimensions = measure_text(&text_to_draw[..], Some(font), Comment::FONT_SIZE, 1.0);
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
            CanvasState::DraggingCanvas { start_offset, .. } => {
                Vec2::distance(*start_offset, self.camera.target.clone()) < CONFIG.click_distance
            }
            _ => false,
        }
    }
    fn start_edit(&mut self, editing_object: Box<dyn CanvasObject>) {
        self.state = CanvasState::Editing {
            cursor: 0,
            editing_object,
        };
        show_keyboard(true);
        // info!(
        //     "Inserted at {:?}",
        //     Vec2::from(mouse_position()) + self.camera.target.clone()
        // );
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
#[derive(Clone)]
enum Tool {
    Equation,
    Comment,
}
struct ToolButton {
    position: Vec2,
    symbol: Texture2D,
    tool: Tool,
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
                    self.canvas.insert_object_if_editing();
                    self.canvas.start_drag();
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
                                //info!("ToolIndex: {}", index);
                                self.canvas.state = CanvasState::Default;
                                self.tool = self.tool_buttons[index].tool.clone();
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

    let equation_font =
        load_ttf_font_from_bytes(include_bytes!("../assets/Symbola_hint.ttf")).unwrap();
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
        tool: Tool::Equation,
    });
    app_state.tool_buttons.push(ToolButton {
        position: Vec2::new(200., 50.),
        symbol: load_texture("assets/comment.png").await.unwrap(),
        tool: Tool::Comment,
    });

    //set_window_position(500, 0);
    // set_fullscreen(true);
    loop {
        clear_background(WHITE);
        app_state.update();
        app_state.draw();
        next_frame().await;
    }
}
