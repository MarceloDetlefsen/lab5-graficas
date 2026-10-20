#![allow(dead_code)]
#![allow(unused_imports)]

use raylib::prelude::*;
use crate::framebuffer::Framebuffer;

pub fn line(
    framebuffer: &mut Framebuffer,
    start: Vector2,
    end: Vector2,
) {
    let mut x0 = start.x.round() as i32;
    let mut y0 = start.y.round() as i32;
    let x1 = end.x.round() as i32;
    let y1 = end.y.round() as i32;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    let mut err = dx - dy;

    loop {
        framebuffer.set_pixel(x0, y0, framebuffer.current_color);

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;

        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }

        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }
}

pub fn draw_poligon(framebuffer: &mut Framebuffer, points: &[Vector2]) {
    if points.len() < 2 {
        return;
    }

    for pair in points.windows(2) {
        line(framebuffer, pair[0], pair[1]);
    }

    line(framebuffer, points[points.len() - 1], points[0]);
}

pub fn triangle(framebuffer: &mut Framebuffer, a: Vector2, b: Vector2, c: Vector2) {
    line(framebuffer, a, b);
    line(framebuffer, b, c);
    line(framebuffer, c, a);
}

pub use crate::framebuffer::point;
