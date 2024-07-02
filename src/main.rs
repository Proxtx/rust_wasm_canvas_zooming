#![feature(let_chains)]

use std::{
    isize,
    ops::{Add, Mul, Sub},
};

use ev::TouchEvent;
use html::Canvas;
use leptos::wasm_bindgen::JsCast;
use leptos::*;
use logging::log;
use rand::Rng;
use std::ops::Deref;
use stylers::style;
use wasm_bindgen::Clamped;
use web_sys::{js_sys::Math::pow, CanvasRenderingContext2d, Element, ImageData, TouchList};

fn main() {
    console_error_panic_hook::set_once();
    let stl = style! {
        .hello {
            background-color: red;
        }
    };
    let (pixel_mat, write_pixel_mat) = create_signal(generate_random_pixel_matrix(100, 100));
    let (width, write_width) =
        create_signal(window().inner_width().unwrap().as_f64().unwrap() as usize);
    let (height, write_height) = create_signal(500);
    mount_to_body(move || {
        view! { class=stl, <PixelView pixels=pixel_mat canvas_width=width canvas_height=height/> }
    })
}

fn generate_random_pixel_matrix(width: usize, height: usize) -> PixelMatrix {
    let mut rng = rand::thread_rng();
    let mut output = Vec::new();
    for x in 0..width {
        output.push(Vec::new());
        for _y in 0..height {
            if x == 0 {
                output[x].push(Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                })
            } else {
                output[x].push(Color {
                    r: rng.gen(),
                    g: rng.gen(),
                    b: rng.gen(),
                    a: 255,
                })
            }
        }
    }

    output
}

type PixelMatrix = Vec<Vec<Color>>;

#[component]
fn PixelView(
    pixels: ReadSignal<PixelMatrix>,
    canvas_width: ReadSignal<usize>,
    canvas_height: ReadSignal<usize>,
) -> impl IntoView {
    let canvas_ref = create_node_ref::<Canvas>();
    let ctx = move || {
        canvas_ref.get().map(|v| {
            v.get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap()
        })
    };

    let (scale, write_scale) = create_signal(4.0);
    let (transform, write_transform) = create_signal(Vector { x: -1.0, y: -1.0 });

    create_effect(move |_| {
        if let Some(ctx) = ctx() {
            ctx.put_image_data(
                &ImageData::new_with_u8_clamped_array(
                    Clamped(&generate_image_data(
                        &pixels(),
                        transform(),
                        scale(),
                        canvas_width(),
                        canvas_height(),
                    )),
                    canvas_width() as u32,
                )
                .unwrap(),
                0.0,
                0.0,
            )
            .unwrap();
        }
    });

    let (last_touches, write_last_touches) = create_signal::<Option<Vec<Vector>>>(None);

    let canvas_start_touches = move |e: TouchEvent| {
        let touches = convert_touch_list_to_canvas_positions(&canvas_ref().unwrap(), &e.touches());
        write_last_touches(Some(touches));
    };

    let canvas_apply_movement = move |e: TouchEvent| {
        let touches = convert_touch_list_to_canvas_positions(&canvas_ref().unwrap(), &e.touches());
        match touches.len() {
            0 => {
                window().alert_with_message("Wtf no sense").unwrap();
            }
            1 => {
                if let Some(v) = last_touches()
                    && let Some(last_touch) = v.first()
                {
                    let movement = touches[0] - *last_touch;
                    write_transform(transform() + movement);
                }
            }
            2 => {
                e.prevent_default();
                if let Some(v) = last_touches()
                    && let Some(first_last_touch) = v.first()
                    && let Some(second_last_touch) = v.get(1)
                {
                    let first_current_touch = touches.first().unwrap();
                    let second_current_touch = touches.get(1).unwrap();

                    let last_distance = *second_last_touch - *first_last_touch;
                    let last_center = *first_last_touch + (last_distance * 0.5);

                    let current_distance = *second_current_touch - *first_current_touch;
                    let current_center = *first_current_touch + (current_distance * 0.5);

                    let diff = current_distance.len() - last_distance.len();
                    let percent_grown = diff / (last_distance.len());

                    let new_scale = scale() * (percent_grown + 1.0);

                    write_transform(
                        transform() - ((current_center - transform()) * (percent_grown))
                            + current_center
                            - last_center,
                    );

                    write_scale(new_scale);
                }
            }
            3.. => {
                window()
                    .alert_with_message("3 Finger zoom not allowed. It's a feature")
                    .unwrap();
            }
        }

        write_last_touches(Some(touches.clone()));
    };

    view! {
        <canvas
            ref=canvas_ref
            width=canvas_width
            height=canvas_height
            on:touchmove=canvas_apply_movement
            on:touchstart=canvas_start_touches
            on:touchend=move |_| { write_last_touches(None) }
        ></canvas>
        <button on:click=move |_| {
            write_scale.update(|v: &mut f32| { *v += 0.1 })
        }>UpScale</button>
        <button on:click=move |_| {
            write_transform.update(|v: &mut Vector| v.x += 1.0)
        }>Left</button>
    }
}

fn convert_touch_list_to_canvas_positions(
    canvas_element: &Element,
    touches: &TouchList,
) -> Vec<Vector> {
    let mut out = Vec::new();
    let client_rect = canvas_element.get_bounding_client_rect();
    for touch_index in 0..touches.length() {
        let touch = touches.get(touch_index).unwrap();
        let hector = Vector {
            y: client_rect.top() as f32 - touch.page_y() as f32,
            x: client_rect.left() as f32 - touch.page_x() as f32,
        };
        out.push(hector);
    }
    out
}

#[derive(Debug, Clone, Copy)]
struct Vector {
    pub x: f32,
    pub y: f32,
}

impl Add for Vector {
    type Output = Vector;
    fn add(self, rhs: Self) -> Self::Output {
        Vector {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vector {
    type Output = Vector;
    fn sub(self, rhs: Self) -> Self::Output {
        Vector {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f32> for Vector {
    type Output = Vector;
    fn mul(self, rhs: f32) -> Self::Output {
        Vector {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Vector {
    fn len(&self) -> f32 {
        (self.x.powf(2.0) + self.y.powf(2.0)).sqrt()
    }
}

#[derive(Debug, Clone, Copy)]
struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

fn generate_image_data(
    pixels: &PixelMatrix,
    transform: Vector,
    scale: f32,
    canvas_width: usize,
    canvas_height: usize,
) -> Vec<u8> {
    let mut image_buffer = vec![0; canvas_width * canvas_height * 4];
    for canvas_x in 0..canvas_width {
        for canvas_y in 0..canvas_height {
            let corresponding_source_pixel = Vector {
                x: ((canvas_x as f32 + transform.x) / scale).floor(),
                y: ((canvas_y as f32 + transform.y) / scale).floor(),
            };
            let (x, y) = (corresponding_source_pixel.x, corresponding_source_pixel.y);
            let color = match (x, y) {
                (..0.0, _) | (_, ..0.0) => &Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 255,
                },
                (x, y) => match pixels.get(x as usize) {
                    Some(v) => match v.get(y as usize) {
                        Some(v) => v,
                        None => &Color {
                            r: 0,
                            g: 0,
                            b: 0,
                            a: 255,
                        },
                    },
                    None => &Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                },
            };
            write_image_buffer(
                &mut image_buffer,
                canvas_width,
                Vector {
                    x: canvas_x as f32,
                    y: canvas_y as f32,
                },
                color,
            )
        }
    }
    image_buffer
}

fn write_image_buffer(buffer: &mut [u8], canvas_width: usize, position: Vector, color: &Color) {
    let start = canvas_width * 4 * position.y.floor() as usize + 4 * position.x.floor() as usize;
    buffer[start] = color.r;
    buffer[start + 1] = color.g;
    buffer[start + 2] = color.b;
    buffer[start + 3] = color.a;
}
