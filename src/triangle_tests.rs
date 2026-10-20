use std::collections::HashSet;

use raylib::prelude::Vector3;

use crate::framebuffer::Framebuffer;
use crate::fragment::Fragment;
use crate::obj::load_obj;
use crate::rasterizer::{TriangleMode, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
use crate::render::fit_to_screen;
use crate::scene::{build_model_matrix, build_triangles};
use crate::triangle::{triangle_barycentric, triangle_scanline};
use crate::vertex::Vertex;
use crate::{render, Uniforms};

fn v(x: f32, y: f32) -> Vertex {
    let mut vertex = Vertex::new_with_color(Vector3::new(x, y, 0.0), Vector3::new(1.0, 1.0, 1.0));
    vertex.transformed_position = Vector3::new(x, y, 0.0);
    vertex
}

fn pixels(fragments: &[Fragment]) -> HashSet<(i32, i32)> {
    fragments.iter().map(|f| (f.position.x.floor() as i32, f.position.y.floor() as i32)).collect()
}

fn compare(a: Vertex, b: Vertex, c: Vertex) {
    let barycentric = triangle_barycentric(&a, &b, &c);
    let scanline = triangle_scanline(&a, &b, &c);
    let bary_pixels = pixels(&barycentric);
    let scan_pixels = pixels(&scanline);
    let extra: HashSet<_> = bary_pixels.difference(&scan_pixels).copied().collect();
    let missing: HashSet<_> = scan_pixels.difference(&bary_pixels).copied().collect();
    assert!(extra.is_empty() && missing.is_empty(), "Píxeles distintos: sobran {:?}, faltan {:?}; fragmentos bary={}, scan={}", extra, missing, barycentric.len(), scanline.len());
    assert_eq!(barycentric.len(), bary_pixels.len(), "Barycentric produjo fragmentos duplicados");
    assert_eq!(scanline.len(), scan_pixels.len(), "Scanline produjo fragmentos duplicados");
    for &(x, y) in bary_pixels.union(&scan_pixels) {
        assert!((0..800).contains(&x) && (0..600).contains(&y), "Píxel fuera de pantalla: ({x}, {y})");
    }
    let mut bary_sorted: Vec<_> = barycentric.iter().collect();
    let mut scan_sorted: Vec<_> = scanline.iter().collect();
    bary_sorted.sort_by_key(|f| (f.position.y as i32, f.position.x as i32));
    scan_sorted.sort_by_key(|f| (f.position.y as i32, f.position.x as i32));
    assert_eq!(bary_sorted.len(), scan_sorted.len());
    for (left, right) in bary_sorted.iter().zip(scan_sorted.iter()) {
        assert_eq!(left.position, right.position);
        assert_eq!(left.color, right.color);
        assert_eq!(left.depth, right.depth);
    }
}

fn compare_orders(a: (f32, f32), b: (f32, f32), c: (f32, f32)) {
    compare(v(a.0, a.1), v(b.0, b.1), v(c.0, c.1));
    let first = pixels(&triangle_barycentric(&v(a.0, a.1), &v(b.0, b.1), &v(c.0, c.1)));
    let reversed = pixels(&triangle_barycentric(&v(c.0, c.1), &v(b.0, b.1), &v(a.0, a.1)));
    assert_eq!(first, reversed, "Invertir el orden cambió los píxeles");
}

#[test]
fn determinista_cubre_los_casos_requeridos() {
    for triangle in [
        ((10.0, 10.0), (14.0, 10.0), (10.0, 14.0)),
        ((50.0, 40.0), (700.0, 120.0), (300.0, 560.0)),
        ((100.0, 100.0), (200.0, 100.0), (150.0, 180.0)),
        ((300.0, 100.0), (300.0, 200.0), (380.0, 150.0)),
        ((100.5, 100.5), (200.5, 100.5), (150.5, 180.5)),
        ((300.5, 100.5), (300.5, 200.5), (380.5, 150.5)),
        ((-50.0, -30.0), (120.0, 40.0), (30.0, 150.0)),
        ((700.0, 500.0), (900.0, 550.0), (750.0, 700.0)),
    ] { compare_orders(triangle.0, triangle.1, triangle.2); }
}

#[test]
fn degenerados_y_fuera_de_pantalla_no_generan_fragmentos() {
    for triangle in [
        ((10.0, 10.0), (50.0, 50.0), (90.0, 90.0)),
        ((10.0, 100.0), (50.0, 100.0), (90.0, 100.0)),
        ((-300.0, -300.0), (-200.0, -300.0), (-250.0, -200.0)),
    ] {
        let (a, b, c) = (v(triangle.0.0, triangle.0.1), v(triangle.1.0, triangle.1.1), v(triangle.2.0, triangle.2.1));
        assert!(triangle_barycentric(&a, &b, &c).is_empty());
        assert!(triangle_scanline(&a, &b, &c).is_empty());
    }
}

#[test]
fn conteo_conocido() {
    let (a, b, c) = (v(0.0, 0.0), v(10.0, 0.0), v(0.0, 10.0));
    assert_eq!(triangle_barycentric(&a, &b, &c).len(), 55);
    assert_eq!(triangle_scanline(&a, &b, &c).len(), 55);
}

fn distance_to_segment(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let (dx, dy) = (bx - ax, by - ay);
    let t = (((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    let (qx, qy) = (ax + t * dx, ay + t * dy);
    ((px - qx).powi(2) + (py - qy).powi(2)).sqrt()
}

fn near_edge(x: i32, y: i32, triangle: [(f32, f32); 3]) -> bool {
    let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
    [(triangle[0], triangle[1]), (triangle[1], triangle[2]), (triangle[2], triangle[0])]
        .into_iter().any(|((ax, ay), (bx, by))| distance_to_segment(px, py, ax, ay, bx, by) < 1e-3)
}

#[test]
fn aleatoria_lcg_solo_tolera_diferencias_de_borde() {
    let mut state = 0x1234_5678_u32;
    let mut next = || { state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223); state as f32 / u32::MAX as f32 };
    let mut total_differences = 0;
    let mut border_differences = 0;
    for _ in 0..2000 {
        let triangle = std::array::from_fn(|_| (-100.0 + next() * 1000.0, -100.0 + next() * 800.0));
        let a = v(triangle[0].0, triangle[0].1); let b = v(triangle[1].0, triangle[1].1); let c = v(triangle[2].0, triangle[2].1);
        let left = pixels(&triangle_barycentric(&a, &b, &c));
        let right = pixels(&triangle_scanline(&a, &b, &c));
        for pixel in left.symmetric_difference(&right) {
            total_differences += 1;
            if near_edge(pixel.0, pixel.1, triangle) { border_differences += 1; }
            else { panic!("Diferencia fuera de borde en {:?} (bary={}, scan={}), triángulo {:?}", pixel, left.contains(pixel), right.contains(pixel), triangle); }
        }
    }
    println!("Diferencias de borde: {border_differences} de {total_differences}");
}

#[test]
fn diagnostico_diferencias_nave_y_overdraw() {
    let obj = load_obj("assets/Nave_X_Y.obj").unwrap();
    let (scale, _) = fit_to_screen(&obj, VIEWPORT_WIDTH as u32, VIEWPORT_HEIGHT as u32, 0.1);
    let uniforms = Uniforms { model_matrix: build_model_matrix(&obj, scale, 0.6, 0.3, 1.0) };
    let triangles = build_triangles(&obj, &uniforms);
    let mut differing_triangles = 0;
    let mut differing_pixels = 0;
    let mut scanline_only = 0;
    let mut barycentric_only = 0;
    let mut max_edge_distance = 0.0_f32;

    for tri in &triangles {
        let bary = pixels(&triangle_barycentric(&tri[0], &tri[1], &tri[2]));
        let scan = pixels(&triangle_scanline(&tri[0], &tri[1], &tri[2]));
        let differences: Vec<_> = bary.symmetric_difference(&scan).copied().collect();
        if !differences.is_empty() {
            differing_triangles += 1;
            differing_pixels += differences.len();
            scanline_only += scan.difference(&bary).count();
            barycentric_only += bary.difference(&scan).count();
            let points = [
                (tri[0].transformed_position.x, tri[0].transformed_position.y),
                (tri[1].transformed_position.x, tri[1].transformed_position.y),
                (tri[2].transformed_position.x, tri[2].transformed_position.y),
            ];
            for (x, y) in differences {
                max_edge_distance = max_edge_distance.max(
                    [(points[0], points[1]), (points[1], points[2]), (points[2], points[0])]
                        .into_iter()
                        .map(|((ax, ay), (bx, by))| distance_to_segment(x as f32 + 0.5, y as f32 + 0.5, ax, ay, bx, by))
                        .fold(f32::INFINITY, f32::min),
                );
            }
        }
    }

    let vertices: Vec<Vertex> = triangles.iter().flat_map(|tri| tri.iter().cloned()).collect();
    let identity = Uniforms { model_matrix: raylib::prelude::Matrix::identity() };
    let mut bary_frame = Framebuffer::new(VIEWPORT_WIDTH as u32, VIEWPORT_HEIGHT as u32, raylib::prelude::Color::new(15, 18, 28, 255));
    let mut scan_frame = Framebuffer::new(VIEWPORT_WIDTH as u32, VIEWPORT_HEIGHT as u32, raylib::prelude::Color::new(15, 18, 28, 255));
    bary_frame.clear();
    scan_frame.clear();
    render(&mut bary_frame, &identity, &vertices, TriangleMode::FilledBarycentric);
    render(&mut scan_frame, &identity, &vertices, TriangleMode::FilledScanline);
    let mut framebuffer_differences = 0;
    for y in 0..VIEWPORT_HEIGHT {
        for x in 0..VIEWPORT_WIDTH {
            if bary_frame.pixel_color(x, y) != scan_frame.pixel_color(x, y) {
                framebuffer_differences += 1;
            }
        }
    }

    println!("Diagnóstico nave: triángulos distintos = {differing_triangles}, píxeles distintos acumulados = {differing_pixels}, solo scanline = {scanline_only}, solo baricéntrico = {barycentric_only}, distancia máxima a arista = {max_edge_distance:.8}, píxeles distintos en framebuffer final = {framebuffer_differences}");
    assert!(max_edge_distance < 1e-3, "La diferencia no está cerca de una arista");
}
