#![allow(dead_code)]

use crate::framebuffer::Framebuffer;
use crate::line::triangle;
use crate::obj::Obj;
use raylib::prelude::{Color, Vector2, Vector3};

pub const LINE_COLOR: Color = Color::new(240, 130, 40, 255);

/// Proyecta un vértice 3D a coordenadas 2D de pantalla usando únicamente sus componentes X e Y.
/// En el espacio de modelado 3D el eje Y positivo apunta hacia arriba, mientras que en coordenadas
/// de pantalla el eje Y crece hacia abajo; por esta razón se invierte el signo de Y (-v.y).
pub fn to_screen(v: Vector3, scale: f32, offset: Vector2) -> Vector2 {
    Vector2::new(
        v.x * scale + offset.x,
        -v.y * scale + offset.y,
    )
}

/// Renderiza en wireframe todos los triángulos de un objeto OBJ proyectados a 2D.
pub fn render_obj(framebuffer: &mut Framebuffer, obj: &Obj, scale: f32, offset: Vector2) {
    framebuffer.set_current_color(LINE_COLOR);

    for chunk in obj.indices.chunks_exact(3) {
        let v0 = obj.vertices[chunk[0]];
        let v1 = obj.vertices[chunk[1]];
        let v2 = obj.vertices[chunk[2]];

        let p0 = to_screen(v0, scale, offset);
        let p1 = to_screen(v1, scale, offset);
        let p2 = to_screen(v2, scale, offset);

        triangle(framebuffer, p0, p1, p2);
    }
}

/// Calcula escala y offset para centrar y ajustar el modelo a la pantalla conservando proporciones.
/// Ocupa como máximo (1.0 - margin) del ancho y alto del framebuffer.
pub fn fit_to_screen(obj: &Obj, width: u32, height: u32, margin: f32) -> (f32, Vector2) {
    let (min, max) = obj.bounds();
    let model_width = max.x - min.x;
    let model_height = max.y - min.y;

    let target_w = width as f32 * (1.0 - margin);
    let target_h = height as f32 * (1.0 - margin);

    let scale_x = if model_width > 0.0 { target_w / model_width } else { 1.0 };
    let scale_y = if model_height > 0.0 { target_h / model_height } else { 1.0 };
    let scale = scale_x.min(scale_y);

    let center_x = (min.x + max.x) / 2.0;
    let center_y = (min.y + max.y) / 2.0;

    let screen_center_x = width as f32 / 2.0;
    let screen_center_y = height as f32 / 2.0;

    // Dado que to_screen aplica:
    //   x_pantalla =  v.x * scale + offset.x  =>  offset.x = screen_center_x - center_x * scale
    //   y_pantalla = -v.y * scale + offset.y  =>  offset.y = screen_center_y + center_y * scale
    let offset = Vector2::new(
        screen_center_x - center_x * scale,
        screen_center_y + center_y * scale,
    );

    (scale, offset)
}
