mod framebuffer;
mod fragment;
mod line;
mod obj;
mod primitive;
mod rasterizer;
mod render;
mod shaders;
mod triangle;
mod vertex;
#[cfg(test)]
mod triangle_tests;

use std::env;
use std::path::Path;
use std::process;
use std::time::Instant;

use framebuffer::Framebuffer;
use obj::load_obj;
use raylib::prelude::*;
use render::{fit_to_screen, render_obj};
use primitive::primitive_assembly;
use rasterizer::{rasterize, TriangleMode, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
use shaders::{fragment_shader, vertex_shader};

pub struct Uniforms {
    pub model_matrix: Matrix,
}

pub fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[vertex::Vertex], mode: TriangleMode) {
    // 1. Procesamiento de vértices.
    let transformed_vertices: Vec<vertex::Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    // 2. Ensamblaje de primitivas.
    let triangles = primitive_assembly(&transformed_vertices);

    // 3. Rasterización.
    let fragments = rasterize(&triangles, mode);

    // 4. Procesamiento de fragmentos.
    for fragment in fragments {
        let fragment_color = fragment_shader(&fragment);
        let color = Color::new(
            (fragment_color.x.clamp(0.0, 1.0) * 255.0) as u8,
            (fragment_color.y.clamp(0.0, 1.0) * 255.0) as u8,
            (fragment_color.z.clamp(0.0, 1.0) * 255.0) as u8,
            255,
        );
        framebuffer.set_pixel(fragment.position.x as i32, fragment.position.y as i32, color);
    }
}

fn render_single_model(obj_path: &str, output_file: &str) {
    let start_load = Instant::now();
    let obj = match load_obj(obj_path) {
        Ok(model) => model,
        Err(err) => {
            eprintln!("Error al cargar el archivo '{}': {}", obj_path, err);
            process::exit(1);
        }
    };
    let load_time = start_load.elapsed();

    // Validación de índices dentro de rango
    if let Some(&max_idx) = obj.indices.iter().max() {
        assert!(
            max_idx < obj.vertices.len(),
            "Error: índice fuera de rango (índice {} para {} vértices)",
            max_idx,
            obj.vertices.len()
        );
    }

    println!("Modelo cargado: '{}'", obj_path);
    println!("  Vértices:   {}", obj.vertices.len());
    println!("  Triángulos: {}", obj.indices.len() / 3);
    println!("  Tiempo:     {:?}", load_time);

    // Framebuffer 800x600 con centrado automático y margen del 10%
    let width = 800;
    let height = 600;
    let (scale, offset) = fit_to_screen(&obj, width, height, 0.1);
    let uniforms = Uniforms {
        model_matrix: Matrix::identity(),
    };

    let mut framebuffer = Framebuffer::new(width, height, Color::new(15, 18, 28, 255));
    framebuffer.clear();

    render_obj(&mut framebuffer, &uniforms, &obj, scale, offset);

    // Asegurar que el directorio de salida exista
    if let Some(parent) = Path::new(output_file).parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    framebuffer.render_to_file(output_file);
    println!("Imagen exportada exitosamente a '{}'\n", output_file);
}

fn show_model_in_window(obj_path: &str) {
    let obj = load_obj(obj_path).expect("No se pudo cargar el modelo OBJ");
    let (scale, _) = fit_to_screen(&obj, VIEWPORT_WIDTH as u32, VIEWPORT_HEIGHT as u32, 0.1);
    let mut framebuffer = Framebuffer::new(VIEWPORT_WIDTH as u32, VIEWPORT_HEIGHT as u32, Color::new(15, 18, 28, 255));
    framebuffer.clear();
    let vertex_array: Vec<vertex::Vertex> = obj.indices.iter().map(|&index| {
        let position = obj.vertices[index];
        vertex::Vertex::new_with_color(position, Vector3::new(1.0, 0.72, 0.4))
    }).collect();
    let (mut window, thread) = raylib::init()
        .size(VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
        .title("Modelo OBJ - Renderer por software")
        .build();
    window.set_target_fps(60);
    let mut mode = TriangleMode::Wireframe;
    let mut yaw = 0.0_f32;
    let mut pitch = 0.0_f32;
    let mut zoom = 1.0_f32;
    let mut auto_yaw = false;
    let mut auto_pitch = false;
    let (min, max) = obj.bounds();
    let model_center = Vector3::new(
        (min.x + max.x) * 0.5,
        (min.y + max.y) * 0.5,
        (min.z + max.z) * 0.5,
    );
    while !window.window_should_close() {
        if window.is_key_pressed(KeyboardKey::KEY_W) {
            mode = TriangleMode::Wireframe;
            println!("Modo activo: {:?}", mode);
        }
        if window.is_key_pressed(KeyboardKey::KEY_B) {
            mode = TriangleMode::FilledBarycentric;
            println!("Modo activo: {:?}", mode);
        }
        if window.is_key_pressed(KeyboardKey::KEY_S) {
            mode = TriangleMode::FilledScanline;
            println!("Modo activo: {:?}", mode);
        }
        let wheel = window.get_mouse_wheel_move();
        zoom = (zoom + wheel * 0.1).clamp(0.2, 4.0);

        if window.is_key_pressed(KeyboardKey::KEY_R) {
            yaw = 0.0;
            pitch = 0.0;
            zoom = 1.0;
            auto_yaw = false;
            auto_pitch = false;
        }
        if window.is_key_pressed(KeyboardKey::KEY_ONE) {
            auto_yaw = !auto_yaw;
        }
        if window.is_key_pressed(KeyboardKey::KEY_TWO) {
            auto_pitch = !auto_pitch;
        }

        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            yaw -= 0.03;
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            yaw += 0.03;
        }
        if window.is_key_down(KeyboardKey::KEY_UP) {
            pitch -= 0.03;
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            pitch += 0.03;
        }
        if auto_yaw {
            yaw += 0.03;
        }
        if auto_pitch {
            pitch += 0.03;
        }

        // Conservamos la geometría 3D y la proyectamos después de rotarla.
        // La traslación se calcula explícitamente para coincidir con la
        // multiplicación Matrix × Vector4 usada por el vertex shader.
        let linear = Matrix::rotate_y(yaw)
            * Matrix::rotate_x(pitch)
            * Matrix::scale(scale * zoom, -scale * zoom, scale * zoom);
        let transformed_center = Vector3::new(
            linear.m0 * model_center.x + linear.m4 * model_center.y + linear.m8 * model_center.z,
            linear.m1 * model_center.x + linear.m5 * model_center.y + linear.m9 * model_center.z,
            linear.m2 * model_center.x + linear.m6 * model_center.y + linear.m10 * model_center.z,
        );
        let mut model_matrix = linear;
        model_matrix.m12 = VIEWPORT_WIDTH as f32 / 2.0 - transformed_center.x;
        model_matrix.m13 = VIEWPORT_HEIGHT as f32 / 2.0 - transformed_center.y;
        model_matrix.m14 = -transformed_center.z;
        let uniforms = Uniforms { model_matrix };
        framebuffer.clear();
        render(&mut framebuffer, &uniforms, &vertex_array, mode);
        framebuffer.draw_in_window(&mut window, &thread);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Modo archivo individual especificado por CLI
        let obj_path = &args[1];
        let output_file = if args.len() > 2 {
            &args[2]
        } else {
            "images/render.png"
        };
        render_single_model(obj_path, output_file);
    } else {
        show_model_in_window("assets/Nave_X_Y.obj");
    }
}
