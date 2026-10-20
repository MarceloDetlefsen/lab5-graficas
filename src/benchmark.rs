use std::hint::black_box;
use std::time::Instant;

use raylib::prelude::Vector3;

use crate::obj::load_obj;
use crate::rasterizer::{rasterize, TriangleMode, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
use crate::scene::{build_model_matrix, build_triangles};
use crate::vertex::Vertex;
use crate::Uniforms;

const REPEATS: usize = 11;
const NAVE_ITERS: usize = 20;
const GRANDES_ITERS: usize = 100;
const THIN_ITERS: usize = 2000;

fn mode_name(mode: TriangleMode) -> &'static str {
    match mode {
        TriangleMode::Wireframe => "Wireframe",
        TriangleMode::FilledBarycentric => "FilledBarycentric",
        TriangleMode::FilledScanline => "FilledScanline",
    }
}

fn large_triangles() -> Vec<[Vertex; 3]> {
    let data = [
        [(20.0, 20.0), (780.0, 40.0), (400.0, 580.0)],
        [(60.0, 560.0), (740.0, 520.0), (400.0, 80.0)],
        [(-100.0, 100.0), (450.0, -80.0), (300.0, 400.0)],
        [(500.0, 250.0), (900.0, 100.0), (850.0, 700.0)],
        [(100.0, 300.0), (350.0, 150.0), (250.0, 500.0)],
        [(500.0, 400.0), (700.0, 200.0), (780.0, 580.0)],
        [(-80.0, 700.0), (200.0, 450.0), (600.0, 720.0)],
        [(150.0, -100.0), (650.0, -50.0), (400.0, 250.0)],
    ];
    data.map(|triangle| {
        triangle.map(|(x, y)| Vertex::new_with_color(
            Vector3::new(x, y, 0.0),
            Vector3::new(1.0, 0.72, 0.4),
        ))
    }).to_vec()
}

fn thin_triangles() -> Vec<[Vertex; 3]> {
    let data = [
        [(-20.0, 0.0), (820.0, 600.0), (820.0, 596.0)],
        [(-20.0, 600.0), (820.0, 0.0), (820.0, 4.0)],
        [(0.0, -20.0), (800.0, 620.0), (796.0, 620.0)],
        [(0.0, 620.0), (800.0, -20.0), (804.0, -20.0)],
        [(-20.0, 200.0), (820.0, 204.0), (820.0, 208.0)],
        [(-20.0, 400.0), (820.0, 396.0), (820.0, 392.0)],
        [(200.0, -20.0), (204.0, 620.0), (208.0, 620.0)],
        [(600.0, -20.0), (596.0, 620.0), (592.0, 620.0)],
    ];
    data.map(|triangle| triangle.map(|(x, y)| Vertex::new_with_color(
        Vector3::new(x, y, 0.0),
        Vector3::new(1.0, 0.72, 0.4),
    ))).to_vec()
}

fn median(values: &mut [u128]) -> u128 {
    values.sort_unstable();
    values[values.len() / 2]
}

fn benchmark_set(name: &str, triangles: &[[Vertex; 3]], iterations: usize, modes: &[TriangleMode]) {
    let mut counts = Vec::new();
    for &mode in modes {
        let warmup = rasterize(black_box(triangles), mode);
        black_box(warmup);
        let measured_count = rasterize(black_box(triangles), mode).len();
        counts.push((mode, black_box(measured_count)));
        let mut measurements = Vec::with_capacity(REPEATS);
        for _ in 0..REPEATS {
            let start = Instant::now();
            for _ in 0..iterations {
                let fragments = rasterize(black_box(triangles), mode);
                black_box(fragments);
            }
            measurements.push(start.elapsed().as_nanos() / iterations as u128);
        }
        let minimum = *measurements.iter().min().unwrap();
        let maximum = *measurements.iter().max().unwrap();
        let med = median(&mut measurements);
        println!("| {name} | {} | {} | {} ns | {} ns | {} ns |", mode_name(mode), measured_count, med, minimum, maximum);
    }
    if counts.iter().filter(|(mode, _)| matches!(mode, TriangleMode::FilledBarycentric | TriangleMode::FilledScanline)).map(|(_, count)| *count).collect::<std::collections::HashSet<_>>().len() > 1 {
        println!("ADVERTENCIA: LOS ALGORITMOS DE RELLENO PRODUCEN CANTIDADES DISTINTAS DE FRAGMENTOS.");
    }
}

pub fn run(selected: Option<TriangleMode>) {
    let modes = selected.map(|mode| vec![mode]).unwrap_or_else(|| vec![TriangleMode::Wireframe, TriangleMode::FilledBarycentric, TriangleMode::FilledScanline]);
    let cpu = std::fs::read_to_string("/proc/cpuinfo").ok()
        .and_then(|info| info.lines().find(|line| line.starts_with("model name")).map(str::to_owned))
        .unwrap_or_else(|| "model name: desconocido".to_string());
    println!("CPU: {}", cpu.split_once(':').map(|(_, value)| value.trim()).unwrap_or(&cpu));
    println!("Release: {}", !cfg!(debug_assertions));
    if cfg!(debug_assertions) { println!("ADVERTENCIA: EJECUTA EL BENCHMARK CON --RELEASE."); }

    let obj = load_obj("assets/Nave_X_Y.obj").expect("No se pudo cargar la nave para benchmark");
    let (scale, _) = crate::render::fit_to_screen(&obj, VIEWPORT_WIDTH as u32, VIEWPORT_HEIGHT as u32, 0.1);
    let uniforms = Uniforms { model_matrix: build_model_matrix(&obj, scale, 0.6, 0.3, 1.0) };
    let nave = build_triangles(&obj, &uniforms);
    let grandes = large_triangles();
    let delgados = thin_triangles();

    println!("\n| Conjunto | Modo | Fragmentos | Mediana por ejecución | Mín | Máx |");
    println!("|---|---|---:|---:|---:|---:|");
    benchmark_set("nave", &nave, NAVE_ITERS, &modes);
    benchmark_set("grandes", &grandes, GRANDES_ITERS, &modes);
    benchmark_set("delgados", &delgados, THIN_ITERS, &modes);
}

pub fn parse_mode(argument: Option<&str>) -> Option<TriangleMode> {
    match argument {
        None => None,
        Some("wireframe") => Some(TriangleMode::Wireframe),
        Some("barycentric") => Some(TriangleMode::FilledBarycentric),
        Some("scanline") => Some(TriangleMode::FilledScanline),
        Some(value) => panic!("Modo de benchmark desconocido: {value}"),
    }
}
