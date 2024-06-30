use std::isize;

use html::Canvas;
use leptos::wasm_bindgen::JsCast;
use leptos::*;
use logging::log;
use rand::Rng;
use stylers::style;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, ImageData};

fn main() {
    console_error_panic_hook::set_once();
    let stl = style! {
        .hello {
            background-color: red;
        }
    };
    let (pixel_mat, write_pixel_mat) = create_signal(generate_random_pixel_matrix(100, 100));
    let (width, write_width) = create_signal(500);
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
            if x == width - 1 {
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
    let (transform, write_transform) = create_signal(Vector { x: -1, y: -1 });

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

    view! {
        <canvas ref=canvas_ref width=canvas_width height=canvas_height></canvas>
        <button on:click=move |_| {
            write_scale.update(|v: &mut f32| { *v += 0.1 })
        }>UpScale</button>
        <button on:click=move |_| {
            write_transform.update(|v: &mut Vector| v.x += 1)
        }>Left</button>
    }
}

#[derive(Debug, Clone, Copy)]
struct Vector {
    pub x: isize,
    pub y: isize,
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
                x: ((canvas_x as isize + transform.x) as f32 / scale).floor() as isize,
                y: ((canvas_y as isize + transform.y) as f32 / scale).floor() as isize,
            };
            let color = match (corresponding_source_pixel.x, corresponding_source_pixel.y) {
                (..=-1, _) | (_, ..=-1) => &Color {
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
                    x: canvas_x as isize,
                    y: canvas_y as isize,
                },
                color,
            )
        }
    }
    image_buffer
}

fn write_image_buffer(buffer: &mut [u8], canvas_width: usize, position: Vector, color: &Color) {
    let start = canvas_width * 4 * position.y as usize + 4 * position.x as usize;
    buffer[start] = color.r;
    buffer[start + 1] = color.g;
    buffer[start + 2] = color.b;
    buffer[start + 3] = color.a;
}
