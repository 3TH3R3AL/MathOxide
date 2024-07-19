/*
*  TODO:
*  - DragnDrop
*  - Select
*  - Copy Paste
*  - Symbols n var evaluation
*  - Functions
*  - Equals
*
*
*
*/
use macroquad::miniquad::window::{quit, show_keyboard};
use macroquad::prelude::*;
//use std::ops::{Add, Sub};
extern crate alloc;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::{self, Display};
use core::str::FromStr;
use simplelog::Config;
extern crate simplelog;
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct Config {
    cursor_blink_rate: u128,
    backspace_interval_initial: f32,
    backspace_interval_ramp: f32,
    click_distance: f32,
    exponentiation_scale_factor: f32,
    subscript_scale_factor: f32,
    division_scale_factor: f32,
    division_padding: (f32, f32),
    equation_font_size: u16,
}
const CONFIG: Config = Config {
    cursor_blink_rate: 500,
    backspace_interval_initial: 0.1,
    backspace_interval_ramp: 0.9,
    click_distance: 2.0,
    exponentiation_scale_factor: 0.8,
    subscript_scale_factor: 0.6,
    division_scale_factor: 0.95,
    division_padding: (5.0, 5.0),
    equation_font_size: 60,
};
const CURSOR_SYMBOL: char = '\u{FF5C}';
const PI_SYMBOL: char = '\u{03C0}';
const THETA_SYMBOL: char = '\u{03B8}';
fn max_f32(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        b
    } else if b.is_nan() {
        a
    } else {
        match a.partial_cmp(&b).unwrap_or(Ordering::Equal) {
            Ordering::Less => b,
            _ => a,
        }
    }
}
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

    fn handle_insertion(&mut self, fonts: &mut Fonts) {}
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
struct TNRenderData {
    w: f32,
    h: f32,
}
impl From<TextDimensions> for TNRenderData {
    fn from(value: TextDimensions) -> Self {
        Self {
            w: value.width,
            h: value.height,
        }
    }
}

impl From<(TextDimensions, f32)> for TNRenderData {
    fn from(value: (TextDimensions, f32)) -> Self {
        Self {
            w: value.0.width,
            h: value.1,
        }
    }
}
#[derive(Clone)]
struct TermNode {
    idx: usize,
    term: Term,
    parent: Option<usize>,
    render_data: Option<TNRenderData>,
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
struct TermTree {
    arena: Vec<TermNode>,
}
impl TermTree {
    fn new() -> Self {
        let init = Vec::from([TermNode {
            idx: 0,
            term: Term::Empty,
            parent: None,
            render_data: None,
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
    fn update_node(&mut self, node: TermNode) {
        self.set(node.idx, node);
    }
    fn set_term(&mut self, idx: usize, term: Term) {
        self.arena[idx].term = term;
    }
    fn set_parent(&mut self, idx: usize, parent: usize) {
        debug_assert!(self.arena.len() > idx);
        self.arena[idx].parent = Some(parent);
    }

    fn set_render_data(&mut self, idx: usize, data: TNRenderData) {
        debug_assert!(self.arena.len() > idx);
        self.arena[idx].render_data = Some(data);
    }

    fn get_render_data(&self, idx: usize) -> TNRenderData {
        let node = self.get(idx);
        node.render_data
            .expect("Data should be filled before gotten")
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
            render_data: None,
        };
        self.arena.push(node);
        self.arena.len() - 1
    }

    fn push_term_np(&mut self, term: Term) -> usize {
        let node = TermNode {
            idx: self.arena.len(),
            parent: None,
            term,
            render_data: None,
        };
        self.arena.push(node);
        self.arena.len() - 1
    }
    // fn form_mult(&mut self, original_term: TermNode, new_term: Term) -> Term {
    //     let new_o_id = self.push(TermNode {
    //         parent: Some(original_term.idx),
    //         ..original_term
    //     });
    //     let new_term_id = self.push_term(new_term, original_term.idx);
    //     Term::Multiplication(Vec::from([new_o_id, new_term_id]))
    // }
    fn replace_child(&mut self, child: TermNode, new_term: Term) -> usize {
        let new_child = self.push_term(new_term, child.parent.unwrap());
        let mut parent = self.get(child.parent.unwrap());
        match parent.term {
            Term::Parentheses(ref mut child_idx) | Term::Negative(ref mut child_idx) => {
                debug_assert!(*child_idx == child.idx);
                *child_idx = new_child;
            }
            Term::Multiplication(ref mut children) | Term::Addition(ref mut children) => {
                let index = children.iter().position(|&x| x == child.idx).unwrap();
                children[index] = new_child;
            }
            Term::Division(ref mut child1, ref mut child2)
            | Term::Exponentiation(ref mut child1, ref mut child2) => {
                if *child1 == child.idx {
                    *child1 = new_child;
                } else if *child2 == child.idx {
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
    fn append_mult(&mut self, original_term: TermNode, new_term: Term) -> usize {
        let mut parent = self.get(original_term.parent.unwrap_or(0)); // Might be a bug idk
        match parent.term {
            Term::Multiplication(ref mut children) => {
                let to_append_location = self.push_term(new_term, parent.idx);
                children.push(to_append_location);
                self.update_node(parent);
                to_append_location
            }
            _ => {
                let new_mult_location = original_term.idx;
                let new_o_location = self.push(original_term);
                self.set_parent(new_o_location, new_mult_location);
                let new_term_location = self.push_term(new_term, new_mult_location);
                self.set_term(
                    new_mult_location,
                    Term::Multiplication(Vec::from([new_o_location, new_term_location])),
                );
                new_term_location
            }
        }
    }
    fn get_mid_line_idx(&self, idx: usize) -> f32 {
        self.get_mid_line(self.get(idx))
    }
    fn get_mid_line(&self, target: TermNode) -> f32 {
        let render_data = target
            .render_data
            .expect("Render data should be filled before midline is called");
        match target.term {
            Term::Division(_num, denom) => render_data.h - self.get_render_data(denom).h,
            Term::Exponentiation(_base, exp) => {
                let child_node_data = self.get_render_data(exp);
                //let base_node_data = self.get_render_data(base);
                child_node_data.h //+ text_height / 2.0 - base_node_data.h - 1.0
            }
            Term::Variable(var_name) => {
                let h = render_data.h;
                if var_name.chars().count() > 2 {
                    h / (1.0 + CONFIG.subscript_scale_factor / 2.0) / 2.0
                } else {
                    h / 2.0
                }
            }
            Term::Multiplication(children) => children
                .into_iter()
                .fold(0.0, |old, new| max_f32(old, self.get_mid_line_idx(new))),
            _ => render_data.h / 2.0,
        }
    }

    fn render_and_fill(
        &mut self,
        target: Option<usize>,
        scale_factor: f32,
        font: Option<&Font>,
        top_left_pos: Vec2,
    ) {
        let current_node = self.get(target.unwrap_or(0));
        if let None = current_node.render_data {
            // info!("Filling Data");
            debug_assert!(target.unwrap_or(0) == 0);
            self.fill_render_data(target, scale_factor, font);
        }
        self.render(target, scale_factor, font, top_left_pos);
    }
    fn render(
        &self,
        target: Option<usize>,
        scale_factor: f32,
        font: Option<&Font>,
        top_left_pos: Vec2,
    ) {
        let font_size = CONFIG.equation_font_size;
        let text_height = measure_text(" | ", font, font_size, scale_factor).height;
        let current_node = self.get(target.unwrap_or(0));
        let current_data = current_node.render_data.unwrap();

        #[cfg(debug_assertions)]
        {
            let box_color = match current_node.term {
                Term::Cursor | Term::Empty | Negative(..) => color_u8!(0, 0, 0, 100),
                Numeral(..) | Variable(..) => color_u8!(164, 14, 26, 100),
                Addition(..) => color_u8!(223, 198, 61, 100),
                Multiplication(..) => color_u8!(53, 130, 184, 100),
                Division(..) => color_u8!(118, 166, 83, 100),
                Parentheses(..) => color_u8!(67, 51, 104, 30),
                Exponentiation(..) => color_u8!(219, 129, 68, 100),
            };
            draw_rectangle(
                top_left_pos.x,
                top_left_pos.y,
                current_data.w,
                current_data.h,
                box_color,
            );
        }
        use Term::*;
        let bottom_left_pos = Vec2::new(top_left_pos.x, top_left_pos.y + current_data.h);
        let text_params = TextParams {
            font_size: CONFIG.equation_font_size,
            font,
            font_scale: scale_factor,
            color: color_u8!(0, 0, 0, 255),
            ..Default::default()
        };
        match current_node.term {
            Empty => {
                draw_text_ex(
                    " | ",
                    top_left_pos.x,
                    top_left_pos.y + text_height,
                    text_params,
                );
                info!("drawing at {}", bottom_left_pos)
            }

            Cursor => {
                let time = instant::now();
                let cursor_visible = (time as u128 / CONFIG.cursor_blink_rate) % 2 == 0;
                if cursor_visible {
                    draw_text_ex(
                        "|",
                        top_left_pos.x,
                        top_left_pos.y + text_height,
                        text_params,
                    )
                }
            }
            Numeral(num, exp) => draw_text_ex(
                &(num as f64 / (u128::pow(10, exp.unwrap_or(0)) as f64)).to_string()[..],
                top_left_pos.x,
                top_left_pos.y + text_height,
                text_params,
            ),

            Variable(mut var_name) => {
                draw_text_ex(
                    &var_name[0..1],
                    top_left_pos.x,
                    top_left_pos.y + text_height,
                    text_params.clone(),
                );
                if var_name.chars().count() > 2 {
                    let time = instant::now();
                    let cursor_visible = (time as u128 / CONFIG.cursor_blink_rate) % 2 == 0;
                    if cursor_visible {
                        var_name = var_name.replace("|", " ");
                    }
                    draw_text_ex(
                        &var_name[2..],
                        top_left_pos.x
                            + measure_text(&var_name[0..1], font, font_size, scale_factor).width,
                        top_left_pos.y + text_height * (1.0 + CONFIG.subscript_scale_factor / 2.0),
                        TextParams {
                            font_scale: scale_factor * CONFIG.subscript_scale_factor,
                            ..text_params
                        },
                    );
                }
            }
            Negative(child) => {
                draw_text_ex("-", bottom_left_pos.x, bottom_left_pos.y, text_params);
                let negative_size = measure_text("-", font, font_size, scale_factor);
                self.render(
                    Some(child),
                    scale_factor,
                    font,
                    Vec2::new(negative_size.width + bottom_left_pos.x, top_left_pos.y),
                );
            }
            Parentheses(child) => {
                let child_node_data = self.get_render_data(child);
                let new_scale_factor = (child_node_data.h / text_height) / scale_factor;
                let new_scale_factor = scale_factor;
                draw_text_ex(
                    "(",
                    bottom_left_pos.x,
                    bottom_left_pos.y,
                    TextParams {
                        font_scale: new_scale_factor,
                        font_scale_aspect: 1.0,
                        ..text_params
                    },
                );
                let parentheses_size = measure_text("(", font, font_size, scale_factor);
                self.render(
                    Some(child),
                    scale_factor,
                    font,
                    Vec2::new(parentheses_size.width + bottom_left_pos.x, top_left_pos.y),
                );
                draw_text_ex(
                    ")",
                    bottom_left_pos.x + parentheses_size.width + child_node_data.w,
                    bottom_left_pos.y,
                    TextParams {
                        font_scale: new_scale_factor,
                        font_scale_aspect: 1.0,
                        ..text_params
                    },
                );
            }
            Addition(children) => {
                let mid_line = children.iter().fold(0.0, |old, child| {
                    max_f32(self.get_mid_line_idx(*child), old)
                });
                // draw_line(
                //     top_left_pos.x,
                //     top_left_pos.y + mid_line,
                //     top_left_pos.x + current_data.w,
                //     top_left_pos.y + mid_line,
                //     scale_factor,
                //     text_params.color,
                // );
                let first = children[0];
                let mut combined_w = 0.0;
                for child_id in children {
                    let child = self.get(child_id);
                    let child_dims = child.render_data.unwrap();

                    if child.idx != first {
                        combined_w += if let Negative(gchild) = child.term {
                            draw_text_ex(
                                " - ",
                                top_left_pos.x + combined_w,
                                top_left_pos.y + mid_line + text_height / 2.0,
                                text_params.clone(),
                            );
                            self.render(
                                Some(gchild),
                                scale_factor,
                                font,
                                Vec2::new(
                                    top_left_pos.x
                                        + measure_text(" - ", font, font_size, scale_factor).width
                                        + combined_w,
                                    top_left_pos.y + mid_line - self.get_mid_line_idx(gchild),
                                ),
                            );
                            measure_text(" - ", font, font_size, scale_factor).width
                                + self.get_render_data(gchild).w // omit negative sign bc
                                                                 // we are doing it here
                        } else {
                            draw_text_ex(
                                " + ",
                                top_left_pos.x + combined_w,
                                top_left_pos.y + mid_line + text_height / 2.0,
                                text_params.clone(),
                            );
                            self.render(
                                Some(child.idx),
                                scale_factor,
                                font,
                                Vec2::new(
                                    top_left_pos.x
                                        + measure_text(" + ", font, font_size, scale_factor).width
                                        + combined_w,
                                    top_left_pos.y + mid_line - self.get_mid_line_idx(child.idx),
                                ),
                            );
                            measure_text(" + ", font, font_size, scale_factor).width + child_dims.w
                        }
                    } else {
                        self.render(
                            Some(child.idx),
                            scale_factor,
                            font,
                            Vec2::new(
                                top_left_pos.x + combined_w,
                                top_left_pos.y + mid_line - self.get_mid_line_idx(child.idx),
                            ),
                        );
                        combined_w += child_dims.w;
                    }
                }
            }

            Multiplication(children) => {
                let mut combined_w = 0.0;
                let mid_line = children.iter().fold(0.0, |old, child| {
                    max_f32(self.get_mid_line_idx(*child), old)
                });
                for child_id in children {
                    let child = self.get(child_id);
                    let child_dims = child.render_data.unwrap();
                    self.render(
                        Some(child.idx),
                        scale_factor,
                        font,
                        Vec2::new(
                            top_left_pos.x + combined_w,
                            top_left_pos.y + mid_line - self.get_mid_line_idx(child.idx),
                        ),
                    );
                    combined_w += child_dims.w;
                }
            }
            Exponentiation(base, child) => {
                let base_node_data = self.get_render_data(base);
                let child_node_data = self.get_render_data(child);
                self.render(
                    Some(base),
                    scale_factor,
                    font,
                    Vec2::new(
                        top_left_pos.x,
                        top_left_pos.y + child_node_data.h + text_height / 2.0
                            - base_node_data.h
                            - 1.0,
                    ),
                );
                self.render(
                    Some(child),
                    scale_factor * CONFIG.exponentiation_scale_factor,
                    font,
                    Vec2::new(top_left_pos.x + base_node_data.w, top_left_pos.y),
                );
            }
            Division(numerator, denominator) => {
                let num_data = self.get_render_data(numerator);
                let denom_data = self.get_render_data(denominator);
                self.render(
                    Some(numerator),
                    scale_factor * CONFIG.division_scale_factor,
                    font,
                    Vec2::new(
                        top_left_pos.x
                            + CONFIG.division_padding.0
                            + max_f32(0.0, denom_data.w - num_data.w) / 2.0,
                        top_left_pos.y,
                    ),
                );
                draw_line(
                    top_left_pos.x,
                    top_left_pos.y + num_data.h + CONFIG.division_padding.1 / 2.0,
                    top_left_pos.x + current_data.w,
                    top_left_pos.y + num_data.h + CONFIG.division_padding.1 / 2.0,
                    1.0,
                    text_params.color,
                );
                self.render(
                    Some(denominator),
                    scale_factor * CONFIG.division_scale_factor,
                    font,
                    Vec2::new(
                        top_left_pos.x
                            + CONFIG.division_padding.0
                            + max_f32(0.0, num_data.w - denom_data.w) / 2.0,
                        top_left_pos.y + CONFIG.division_padding.1 + num_data.h,
                    ),
                );
            }
        };
    }
    fn fill_render_data(&mut self, target: Option<usize>, scale_factor: f32, font: Option<&Font>) {
        use Term::*;
        let font_size = CONFIG.equation_font_size;
        let text_height = measure_text(" | ", font, font_size, scale_factor).height;
        let current_node = self.get(target.unwrap_or(0));
        let new_render_data = match current_node.term {
            Empty => TNRenderData::from(measure_text(" | ", font, font_size, scale_factor)),
            Cursor => TNRenderData::from(measure_text("|", font, font_size, scale_factor)),
            Numeral(num, exp) => TNRenderData::from((
                measure_text(
                    &(num as f64 / (u128::pow(10, exp.unwrap_or(0)) as f64)).to_string()[..],
                    font,
                    font_size,
                    scale_factor,
                ),
                text_height,
            )),
            Variable(var_name) => {
                let mut base = TNRenderData::from((
                    measure_text(&var_name[0..1], font, font_size, scale_factor),
                    text_height,
                ));
                if var_name.chars().count() > 2 {
                    base.h += text_height * CONFIG.subscript_scale_factor / 2.0;
                    base.w += measure_text(
                        &var_name[2..],
                        font,
                        font_size,
                        scale_factor * CONFIG.subscript_scale_factor,
                    )
                    .width;
                }
                base
            }
            Negative(child) => {
                let negative_size = measure_text("-", font, font_size, scale_factor);
                self.fill_render_data(Some(child), scale_factor, font);
                let child_node_data = self.get_render_data(child);
                TNRenderData {
                    w: negative_size.width + child_node_data.w,
                    h: max_f32(negative_size.height, child_node_data.h),
                }
            }
            Parentheses(child) => {
                let parentheses_size = measure_text("()", font, font_size, scale_factor);
                self.fill_render_data(Some(child), scale_factor, font);
                let child_node_data = self.get_render_data(child);
                TNRenderData {
                    w: parentheses_size.width + child_node_data.w,
                    h: max_f32(parentheses_size.height, child_node_data.h),
                }
            }
            Addition(children) => {
                let mut dims = TNRenderData { w: 0.0, h: 0.0 };
                let first = children[0];
                for child_id in children {
                    self.fill_render_data(Some(child_id), scale_factor, font);
                    let child = self.get(child_id);
                    let child_dims = child.render_data.unwrap();

                    if child.idx != first {
                        dims.w += if let Negative(gchild) = child.term {
                            measure_text(" - ", font, font_size, scale_factor).width
                                + self.get_render_data(gchild).w // omit negative sign bc
                                                                 // we are doing it here
                        } else {
                            measure_text(" + ", font, font_size, scale_factor).width + child_dims.w
                        }
                    } else {
                        dims.w += if let Negative(gchild) = child.term {
                            self.get_render_data(gchild).w
                        } else {
                            child_dims.w
                        }
                    }
                    dims.h = max_f32(dims.h, child_dims.h);
                }
                dims
            }

            Multiplication(children) => {
                let mut dims = TNRenderData { w: 0.0, h: 0.0 };
                for child_id in children {
                    self.fill_render_data(Some(child_id), scale_factor, font);
                    let child = self.get(child_id);
                    let child_dims = child.render_data.unwrap();
                    dims.w += child_dims.w;
                    dims.h = max_f32(dims.h, child_dims.h);
                }
                dims
            }
            Exponentiation(base, child) => {
                self.fill_render_data(Some(base), scale_factor, font);
                self.fill_render_data(
                    Some(child),
                    scale_factor * CONFIG.exponentiation_scale_factor,
                    font,
                );
                let child_node_data = self.get_render_data(child);
                let base_node_data = self.get_render_data(base);
                TNRenderData {
                    w: base_node_data.w + child_node_data.w,
                    h: child_node_data.h + text_height / 2.0 - 1.0,
                }
            }

            Division(numerator, denominator) => {
                self.fill_render_data(
                    Some(numerator),
                    scale_factor * CONFIG.division_scale_factor,
                    font,
                );
                self.fill_render_data(
                    Some(denominator),
                    scale_factor * CONFIG.division_scale_factor,
                    font,
                );
                let num_data = self.get_render_data(numerator);
                let denom_data = self.get_render_data(denominator);
                TNRenderData {
                    w: max_f32(num_data.w, denom_data.w) + CONFIG.division_padding.0 * 2.0,
                    h: denom_data.h + num_data.h + CONFIG.division_padding.1,
                }
            }
        };

        self.set_render_data(target.unwrap_or(0), new_render_data);
        debug_assert!(self.get(target.unwrap_or(0)).render_data.is_some())
    }

    fn evaluate_eq(&self, target: Option<usize>) -> f32 {
        use Term::*;
        let current_node = self.get(target.unwrap_or(0));
        match current_node.term {
            Empty => 1.0,
            Cursor => 1.0,
            Numeral(num, exp) => (num as f32 / (u128::pow(10, exp.unwrap_or(0)) as f32)),
            Variable(var_name) => 1.0,
            Negative(child) => -self.evaluate_eq(Some(child)),
            Parentheses(child) => self.evaluate_eq(Some(child)),
            Addition(children) => children
                .into_iter()
                .fold(0.0, |old, new| old + self.evaluate_eq(Some(new))),
            Multiplication(children) => children
                .into_iter()
                .fold(1.0, |old, new| old * self.evaluate_eq(Some(new))),
            Exponentiation(base, child) => self
                .evaluate_eq(Some(base))
                .powf(self.evaluate_eq(Some(child))),
            Division(numerator, denominator) => {
                self.evaluate_eq(Some(numerator)) / self.evaluate_eq(Some(denominator))
            }
        }
    }
}
impl FromStr for TermTree {
    type Err = ParseTermError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Term::*;
        let mut root = TermTree::new();
        let mut empty_term = Some(0);
        let mut adding_term: Option<usize> = None;
        let mut chars = s.chars();

        for char in chars {
            debug_assert!(empty_term.is_none() || adding_term.is_none());
            if let Some(cursor) = empty_term {
                let current_term = root.get(cursor);
                debug_assert!(if let Empty = current_term.term {
                    true
                } else {
                    false
                });
                let new_node = TermNode {
                    term: match char {
                        '0'..='9' => Numeral(char.to_digit(10).unwrap() as u128, None),
                        'a'..='z' | THETA_SYMBOL | PI_SYMBOL | 'A'..='Z' => {
                            Variable(String::from(char))
                        }

                        '(' => Parentheses(root.push_term(Empty, cursor)),
                        '-' => Negative(root.push_term(Empty, cursor)),
                        CURSOR_SYMBOL => Cursor,
                        _ => {
                            return Err(ParseTermError);
                        }
                    },
                    ..current_term
                };
                (empty_term, adding_term) = match char {
                    CURSOR_SYMBOL | '0'..='9' | 'a'..='z' | 'A'..='Z' => (None, Some(cursor)),
                    '(' | '-' => {
                        if let Parentheses(empty) = new_node.term {
                            (Some(empty), None)
                        } else if let Negative(empty) = new_node.term {
                            (Some(empty), None)
                        } else {
                            return Err(ParseTermError);
                        }
                    }
                    _ => {
                        return Err(ParseTermError);
                    }
                };
                root.set(cursor, new_node);
                continue;
            }
            if let Some(cursor) = adding_term {
                let current_term = root.get(cursor);
                match (current_term.term.clone(), char) {
                    (Numeral(current, None), '0'..='9') => root.set_term(
                        cursor,
                        Numeral(current * 10 + char.to_digit(10).unwrap() as u128, None),
                    ),

                    (Numeral(current, None), '.') => {
                        root.set_term(cursor, Numeral(current, Some(0)))
                    }
                    (Numeral(current, Some(exp)), '0'..='9') => root.set_term(
                        cursor,
                        Numeral(
                            current * 10 + char.to_digit(10).unwrap() as u128,
                            Some(exp + 1),
                        ),
                    ),
                    (
                        Variable(var_name),
                        '_' | 'a'..='z' | 'A'..='Z' | THETA_SYMBOL | PI_SYMBOL,
                    ) => {
                        if var_name.chars().count() > 1 || char == '_' {
                            root.set_term(cursor, Variable(format!("{}{}", var_name, char)));
                        } else {
                            let new_var_term =
                                root.append_mult(current_term, Variable(char.to_string()));
                            adding_term = Some(new_var_term);
                        }
                    }
                    (Variable(var_name), CURSOR_SYMBOL) if var_name.len() > 1 => {
                        root.set_term(cursor, Variable(format!("{}{}", var_name, "|")));
                    }
                    (Variable(var_name), '0'..='9') if var_name.len() > 1 => {
                        let new_num_term = root.append_mult(
                            current_term,
                            Numeral(char.to_digit(10).unwrap() as u128, None),
                        );
                        adding_term = Some(new_num_term);
                    }
                    (_, ' ') => {
                        // NOTE: This does not work to escape extended variable names
                        // when in root. May fix later, but not a large problem
                        let mut current_term = current_term;
                        let mut guard = 0;
                        loop {
                            guard += 1;
                            if guard > 20 {
                                panic!("Infinite Loop");
                            }
                            let parent = match current_term.parent {
                                Some(idx) => root.get(idx),
                                None => TermNode {
                                    idx: 0,
                                    term: Division(0, 0),
                                    parent: None,
                                    render_data: None,
                                },
                            };
                            match parent.term {
                                Division(..) | Exponentiation(..) => {
                                    adding_term = Some(parent.idx);
                                    break;
                                }
                                _ => {
                                    current_term = parent;
                                }
                            }
                        }
                    }
                    (_, CURSOR_SYMBOL) => {
                        let new_var_term = root.append_mult(current_term, Cursor);
                        adding_term = Some(new_var_term);
                    }
                    (_, '0'..='9') => {
                        let new_num_term = root.append_mult(
                            current_term,
                            Numeral(char.to_digit(10).unwrap() as u128, None),
                        );
                        adding_term = Some(new_num_term);
                    }
                    (_, 'a'..='z' | 'A'..='Z') => {
                        let new_var_term =
                            root.append_mult(current_term, Variable(char.to_string()));
                        adding_term = Some(new_var_term);
                    }
                    (_, '(') => {
                        let new_empty_term = root.push_term_np(Empty);
                        let new_var_term =
                            root.append_mult(current_term, Parentheses(new_empty_term));
                        root.set_parent(new_empty_term, new_var_term);
                        adding_term = None;
                        empty_term = Some(new_empty_term);
                    }
                    (Numeral(..) | Variable(..) | Parentheses(..) | Cursor, '^') => {
                        // info!("expo attempted")
                        let new_empty = root.push_term(Empty, cursor);
                        let new_term = Exponentiation(
                            root.push_term(current_term.term.clone(), cursor),
                            new_empty,
                        );
                        root.set_term(cursor, new_term);
                        empty_term = Some(new_empty);
                        adding_term = None;
                    }
                    (_, ')') => {
                        let mut current = current_term;
                        let mut guard = 0;
                        loop {
                            guard += 1;
                            if guard > 20 {
                                panic!("Infinite Loop");
                            }
                            if let Some(parent) = current.parent {
                                current = root.get(parent);
                                if let Parentheses(..) = current.term {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        empty_term = None;
                        adding_term = Some(current.idx);
                    }
                    (_, '/') => {
                        let denominator_location = root.push_term(Empty, cursor);
                        let mut numerator = current_term;

                        let mut guard = 0;
                        loop {
                            guard += 1;
                            if guard > 20 {
                                panic!("Infinite Loop");
                            }
                            if let Some(parent) = numerator.parent {
                                let next_parent = root.get(parent);
                                if let Multiplication(..) = next_parent.term {
                                    numerator = next_parent;
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        let new_term = Division(
                            root.push_term(numerator.term.clone(), cursor),
                            denominator_location,
                        );
                        root.set_term(numerator.idx, new_term);
                        empty_term = Some(denominator_location);
                        adding_term = None;
                    }
                    (_, '+' | '-') => {
                        let mut current_term = current_term;
                        let mut guard = 0;
                        loop {
                            guard += 1;
                            if guard > 20 {
                                panic!("Infinite Loop");
                            }
                            let mut parent = match current_term.parent {
                                Some(idx) => root.get(idx),
                                None => TermNode {
                                    idx: 0,
                                    term: Empty,
                                    parent: None,
                                    render_data: None,
                                },
                            };
                            match parent.term {
                                Addition(ref mut children) => {
                                    let to_append_location = match char {
                                        '+' => {
                                            let to_append_location =
                                                root.push_term(Empty, parent.idx);
                                            children.push(to_append_location);
                                            root.update_node(parent);
                                            to_append_location
                                        }
                                        '-' => {
                                            let new_empty_location = root.push_term_np(Empty);
                                            let to_append_location = root.push_term(
                                                Negative(new_empty_location),
                                                parent.idx,
                                            );

                                            root.set_parent(new_empty_location, to_append_location);
                                            children.push(to_append_location);
                                            root.update_node(parent);
                                            new_empty_location
                                        }
                                        _ => {
                                            panic!("Invalid char")
                                        }
                                    };
                                    empty_term = Some(to_append_location);
                                    break;
                                }
                                Negative(..) => {
                                    current_term = parent;
                                }
                                Multiplication(..) => {
                                    current_term = parent;
                                }
                                _ => {
                                    let new_term_location = match char {
                                        '+' => {
                                            let new_mult_location = current_term.idx;
                                            let new_o_location = root.push(current_term);
                                            root.set_parent(new_o_location, new_mult_location);
                                            let new_term_location =
                                                root.push_term(Empty, new_mult_location);
                                            root.set_term(
                                                new_mult_location,
                                                Addition(Vec::from([
                                                    new_o_location,
                                                    new_term_location,
                                                ])),
                                            );
                                            new_term_location
                                        }
                                        '-' => {
                                            let new_mult_location = current_term.idx;
                                            let new_o_location = root.push(current_term);
                                            root.set_parent(new_o_location, new_mult_location);

                                            let new_empty_location = root.push_term_np(Empty);
                                            let to_append_location = root.push_term(
                                                Negative(new_empty_location),
                                                new_mult_location,
                                            );
                                            root.set_parent(new_empty_location, to_append_location);

                                            root.set_term(
                                                new_mult_location,
                                                Addition(Vec::from([
                                                    new_o_location,
                                                    to_append_location,
                                                ])),
                                            );
                                            new_empty_location
                                        }
                                        _ => {
                                            panic!("Invalid char")
                                        }
                                    };
                                    empty_term = Some(new_term_location);
                                    break;
                                }
                            }
                        }
                        adding_term = None;
                    }

                    _ => {
                        return Err(ParseTermError);
                    }
                };
            }
        }
        Ok(root)
    }
}

impl TermNode {
    fn to_string(&self, tree: &TermTree) -> String {
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

impl Display for TermTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TermTree:\n{} = {},",
            self.get(0).to_string(self),
            self.evaluate_eq(None)
        )
    }
}
/// A Mathematical Equation on the Canvas
struct Equation {
    text: String,
    pos: Vec2,
    equation: Vec<TermTree>,
}

impl Equation {
    fn replace_symbols(&self) -> String {
        Equation::replace_symbols_str(&self.text)
    }
    fn replace_symbols_str(text: &String) -> String {
        text.replace("pi", "\u{03C0}")
            .replace("theta", "\u{03B8}")
            .replace("*", "\u{00B7}")
    }
    fn parse_text(&self) -> Result<Vec<TermTree>, ParseTermError> {
        let split = self.text.split("=");
        let mut ret = Vec::new();
        for str in split {
            let term = TermTree::from_str(str)?;
            ret.push(term);
        }
        Ok(ret)
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
        // self.text = self.replace_symbols();
    }
    fn backspace(&mut self, cursor: usize) {
        self.text.remove(cursor - 1);
        // let mut chars: Vec<char> = self.text.chars().collect();
        // if cursor < chars.len() {
        //     chars.remove(cursor);
        // }
        // self.text = chars.into_iter().collect();
    }
    fn handle_insertion(&mut self, fonts: &mut Fonts) {
        let font = &mut fonts.equations;
        let equation = self.parse_text();
        self.equation = match equation {
            Ok(mut equation) => {
                let mut eq_to_set = Vec::new();
                for mut term in equation {
                    let current_node = term.get(0);
                    if let None = current_node.render_data {
                        term.fill_render_data(None, 1.0, Some(font));
                    }
                    eq_to_set.push(term);
                }
                eq_to_set
            }
            Err(_) => Vec::new(),
        }
    }
    fn draw(&self, fonts: &mut Fonts) {
        let font = &mut fonts.equations;
        let mut combined_w = 0.0;
        let midline = self
            .equation
            .iter()
            .fold(0.0, |old, new| max_f32(old, new.get_mid_line_idx(0)));
        let term_tree = match self.equation.get(0) {
            Some(t) => t,
            None => return,
        };
        let c_midline = term_tree.get_mid_line_idx(0);
        term_tree.render(
            None,
            1.0,
            Some(font),
            Vec2::new(self.pos.x + combined_w, self.pos.y + midline - c_midline),
        );
        combined_w += term_tree.get_render_data(0).w;
        for term_tree in &self.equation[1..] {
            let eq_sign_size = measure_text(" = ", Some(&font), CONFIG.equation_font_size, 1.0);
            draw_text(
                " = ",
                self.pos.x + combined_w,
                self.pos.y + midline - eq_sign_size.height / 2.0,
                CONFIG.equation_font_size as f32,
                color_u8!(0, 0, 0, 255),
            );
            combined_w += eq_sign_size.width;
            let c_midline = term_tree.get_mid_line_idx(0);
            term_tree.render(
                None,
                1.0,
                Some(font),
                Vec2::new(self.pos.x + combined_w, self.pos.y + midline - c_midline),
            );
            combined_w += term_tree.get_render_data(0).w;
        }
    }
    fn edit_draw(&self, cursor: usize, fonts: &mut Fonts) {
        let font = &mut fonts.equations;
        let mut text_to_draw = self.text.clone();
        text_to_draw.insert(cursor, CURSOR_SYMBOL);
        let equation_tree = TermTree::from_str(&text_to_draw[..]);
        match equation_tree {
            Ok(mut tree) => {
                info!("{}", tree);
                tree.render_and_fill(None, 1.0, Some(font), self.pos);
            }
            Err(_) => info!("Parsing Error :("),
        }
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
    const LINE_COLOR: Color = color_u8!(0, 0, 0, 100);
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

    fn insert_object_if_editing(&mut self, fonts: &mut Fonts) {
        let temp_state = core::mem::replace(&mut self.state, CanvasState::Default);
        match temp_state {
            CanvasState::Editing { editing_object, .. } => {
                if !editing_object.is_empty() {
                    self.objects.push(editing_object);
                    let idx = self.objects.len() - 1;
                    self.objects[idx].handle_insertion(fonts);
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
                    self.canvas.insert_object_if_editing(&mut self.fonts);
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
                                        equation_tree: None,
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
