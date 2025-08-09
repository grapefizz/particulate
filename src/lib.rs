//! Falling-sand physics engine for WASM + HTML5 canvas

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, window};
use std::cell::RefCell;
use std::rc::Rc;

const WIDTH: usize = 200;
const HEIGHT: usize = 150;
const CELL_SIZE: usize = 4; // Each cell is drawn as a 4x4 block

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[derive(Copy, Clone, PartialEq)]
pub enum Cell {
    Empty,
    Sand,
}

pub struct World {
    pub grid: [[Cell; WIDTH]; HEIGHT],
}

impl World {
    pub fn new() -> Self {
        Self { grid: [[Cell::Empty; WIDTH]; HEIGHT] }
    }

    pub fn step(&mut self) {
        // Update from bottom up to avoid double-moving sand
        for y in (0..HEIGHT-1).rev() {
            for x in 0..WIDTH {
                if self.grid[y][x] == Cell::Sand {
                    // Try to move down
                    if self.grid[y+1][x] == Cell::Empty {
                        self.grid[y+1][x] = Cell::Sand;
                        self.grid[y][x] = Cell::Empty;
                    } else {
                        // Try diagonals
                        let mut moved = false;
                        if x > 0 && self.grid[y+1][x-1] == Cell::Empty {
                            self.grid[y+1][x-1] = Cell::Sand;
                            self.grid[y][x] = Cell::Empty;
                            moved = true;
                        } else if x+1 < WIDTH && self.grid[y+1][x+1] == Cell::Empty {
                            self.grid[y+1][x+1] = Cell::Sand;
                            self.grid[y][x] = Cell::Empty;
                            moved = true;
                        }
                        if moved {
                            continue;
                        }
                    }
                }
            }
        }
    }

    pub fn set_sand(&mut self, x: usize, y: usize) {
        if x < WIDTH && y < HEIGHT {
            self.grid[y][x] = Cell::Sand;
        }
    }
}

thread_local! {
    static WORLD: RefCell<World> = RefCell::new(World::new());
    static MOUSE_DOWN: RefCell<bool> = RefCell::new(false);
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    let win = window().unwrap();
    let document = win.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: HtmlCanvasElement = canvas.dyn_into().unwrap();
    let context = canvas
        .get_context("2d").unwrap().unwrap()
        .dyn_into::<CanvasRenderingContext2d>().unwrap();

    // Mouse events
    {
        let canvas_ref = canvas.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            MOUSE_DOWN.with(|down| *down.borrow_mut() = true);
            paint_at_mouse(&canvas_ref, &event);
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }
    {
        let canvas_ref = canvas.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            if MOUSE_DOWN.with(|down| *down.borrow()) {
                paint_at_mouse(&canvas_ref, &event);
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }
    {
        let closure = Closure::wrap(Box::new(move |_event: MouseEvent| {
            MOUSE_DOWN.with(|down| *down.borrow_mut() = false);
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref()).unwrap();
        canvas.add_event_listener_with_callback("mouseleave", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    // Animation loop
    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();
    let context = Rc::new(context);
    let canvas = Rc::new(canvas);
    let win2 = win.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // Step simulation
        WORLD.with(|w| w.borrow_mut().step());
        // Render
        render(&context, &canvas);
        // Schedule next frame
        win2
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));
    win
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}

fn paint_at_mouse(canvas: &HtmlCanvasElement, event: &MouseEvent) {
    let element: &web_sys::Element = canvas.as_ref();
    let rect = element.get_bounding_client_rect();
    let scale_x = canvas.width() as f64 / rect.width();
    let scale_y = canvas.height() as f64 / rect.height();
    let x = ((event.client_x() as f64 - rect.left()) * scale_x) as usize / CELL_SIZE;
    let y = ((event.client_y() as f64 - rect.top()) * scale_y) as usize / CELL_SIZE;
    WORLD.with(|w| w.borrow_mut().set_sand(x, y));
}

fn render(ctx: &CanvasRenderingContext2d, canvas: &HtmlCanvasElement) {
    ctx.set_fill_style_str("black");
    ctx.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    WORLD.with(|w| {
        let w = w.borrow();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                match w.grid[y][x] {
                    Cell::Sand => {
                        ctx.set_fill_style_str("#e2c275");
                        ctx.fill_rect(
                            (x * CELL_SIZE) as f64,
                            (y * CELL_SIZE) as f64,
                            CELL_SIZE as f64,
                            CELL_SIZE as f64,
                        );
                    }
                    _ => {}
                }
            }
        }
    });
}
