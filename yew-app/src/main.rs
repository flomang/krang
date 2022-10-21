use std::cell::RefCell;
use std::rc::Rc;

use std::f64;
use std::f64::consts::PI;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;
use web_sys::{window, HtmlCanvasElement};
use yew::{html, Component, Context, Html, NodeRef};
use lib_simulation as sim;
use rand::prelude::*;

// Wrap gl in Rc (Arc for multi-threaded) so it can be injected into the render-loop closure.
pub struct App {
    node_ref: NodeRef,
}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <canvas id="viewport" width="800" height="800" ref={self.node_ref.clone()} />
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        // Only start the render loop if it's the first render
        // There's no loop cancellation taking place, so if multiple renders happen,
        // there would be multiple loops running. That doesn't *really* matter here because
        // there's no props update and no SSR is taking place, but it is something to keep in
        // consideration
        if !first_render {
            return;
        }
        // Once rendered, store references for the canvas and GL context. These can be used for
        // resizing the rendering area when the window or canvas element are resized, as well as
        // for making GL calls.
        let _document = web_sys::window().unwrap().document().unwrap();
        let window = web_sys::window().unwrap();

        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
        let viewport_width = canvas.width();
        let viewport_height = canvas.height();
        let viewport_scale = window.device_pixel_ratio();

        canvas.set_width(viewport_width * viewport_scale as u32);
        canvas.set_height(viewport_height * viewport_scale as u32);

        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        self.render(context, viewport_width as f64, viewport_height as f64);
    }
}

impl App {
    fn request_animation_frame(f: &Closure<dyn FnMut()>) {
        window()
            .unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .expect("should register `requestAnimationFrame` OK");
    }

    fn draw_triangle(context: &CanvasRenderingContext2d, x: f64, y: f64, size: f64, rotation: f64) {
        context.begin_path();

        context.move_to(
            x + rotation.cos() * size * 1.5,
            y + rotation.sin() * size * 1.5,
        );

        context.line_to(
            x + (rotation + 2.0 / 3.0 * PI).cos() * size,
            y + (rotation + 2.0 / 3.0 * PI).sin() * size,
        );

        context.line_to(
            x + (rotation + 4.0 / 3.0 * PI).cos() * size,
            y + (rotation + 4.0 / 3.0 * PI).sin() * size,
        );

        context.line_to(
            x + rotation.cos() * size * 1.5,
            y + rotation.sin() * size * 1.5,
        );

        context.set_fill_style(&JsValue::from_str("rgb(255, 255, 255)"));
        context.fill();
    }

    fn draw_circle(context: &CanvasRenderingContext2d, x: f64, y: f64, radius: f64) {
        context.begin_path();

        // ---
        // | Circle's center.
        // ----- v -v
        let _ = context.arc(x, y, radius, 0.0, 2.0 * std::f64::consts::PI);
        // ------------------- ^ -^-----------^
        // | Range at which the circle starts and ends, in radians.
        // |
        // | By manipulating these two parameters you can e.g. draw
        // | only half of a circle, Pac-Man style.
        // ---
        context.set_fill_style(&JsValue::from_str("rgb(0, 255, 128)"));
        context.fill();
    }

    fn render(&mut self, context: CanvasRenderingContext2d, view_width: f64, view_height: f64) {
        let mut rng = thread_rng();
        let mut sim = sim::Simulation::random(&mut rng);
        let cb = Rc::new(RefCell::new(None));

        *cb.borrow_mut() = Some(Closure::wrap(Box::new({
            let cb = cb.clone();
            move || {
                context.clear_rect(0.0, 0.0, view_width, view_width);

                sim.step(&mut rng);

                let world = sim.world();

                for food in world.foods() {
                    let pos = food.position();

                    Self::draw_circle(
                        &context,
                        pos.x as f64 * view_width,
                        pos.y as f64 * view_height,
                        (0.01 / 2.0) * view_width,
                    )
                }

                for animal in world.animals() {
                    let pos = animal.position();

                    Self::draw_triangle(
                        &context,
                        pos.x as f64 * view_width,
                        pos.y as f64 * view_height,
                        0.01 * view_width,
                        animal.rotation().angle() as f64,
                    )
                }

                App::request_animation_frame(cb.borrow().as_ref().unwrap());
            }
        }) as Box<dyn FnMut()>));

        App::request_animation_frame(cb.borrow().as_ref().unwrap());
    }
}

fn main() {
    yew::start_app::<App>();
}
