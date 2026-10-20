mod framebuffer;
mod line;
mod obj;
mod render;

use std::env;
use std::path::Path;
use std::process;
use std::time::Instant;

use framebuffer::Framebuffer;
use obj::load_obj;
use raylib::prelude::*;
use render::{fit_to_screen, render_obj};

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

    let mut framebuffer = Framebuffer::new(width, height, Color::new(15, 18, 28, 255));
    framebuffer.clear();

    render_obj(&mut framebuffer, &obj, scale, offset);

    // Asegurar que el directorio de salida exista
    if let Some(parent) = Path::new(output_file).parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    framebuffer.render_to_file(output_file);
    println!("Imagen exportada exitosamente a '{}'\n", output_file);
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
        // Modo por defecto: Renderizar las 6 variantes de planos de la nave
        let default_targets = [
            ("assets/Nave_X_Y.obj", "images/render_X_Y.png"),
            ("assets/Nave_-X_Y.obj", "images/render_-X_Y.png"),
            ("assets/Nave_Z_Y.obj", "images/render_Z_Y.png"),
            ("assets/Nave_-Z_Y.obj", "images/render_-Z_Y.png"),
            ("assets/Nave_X_Z.obj", "images/render_X_Z.png"),
            ("assets/Nave_X_-Z.obj", "images/render_X_-Z.png"),
        ];

        let mut rendered_any = false;
        for (obj_path, out_img) in &default_targets {
            if Path::new(obj_path).exists() {
                render_single_model(obj_path, out_img);
                rendered_any = true;
            }
        }

        // Si existe images/render_X_Y.png, guardamos también como images/render.png por compatibilidad
        if Path::new("images/render_X_Y.png").exists() {
            let _ = std::fs::copy("images/render_X_Y.png", "images/render.png");
        }

        if !rendered_any {
            eprintln!("No se encontraron modelos en assets/ para renderizar.");
            process::exit(1);
        }
    }
}
