# Laboratorio 5: Rasterización de triángulos

Renderer por software en Rust 2024. Carga modelos OBJ manualmente, procesa vértices, ensambla triángulos, genera fragmentos y escribe en un framebuffer propio. Raylib se usa solo para tipos matemáticos, `Image` y la ventana.

Autor: **Marcelo Detlefsen** — Universidad del Valle de Guatemala, Gráficas por Computadora.

## Cómo ejecutar

```bash
cargo run --release
cargo run --release -- assets/Nave_X_Y.obj images/salida.png
cargo run --release -- --export-modes
cargo run --release -- --benchmark-triangles
cargo test
```

La ventana usa un framebuffer de `800x600` y muestra `assets/Nave_X_Y.obj`.

Controles: flechas izquierda/derecha rotan horizontalmente; arriba/abajo rotan verticalmente; la rueda hace zoom; `1` y `2` activan rotación automática horizontal y vertical; `R` reinicia cámara, zoom y movimiento automático; `W`, `B` y `S` seleccionan wireframe, relleno baricéntrico y relleno scanline.

## Pipeline

```text
OBJ → vertex shader → primitive assembly → rasterización → fragment shader → framebuffer
```

El vertex shader aplica la matriz de modelo. `primitive_assembly` agrupa la secuencia en `Vec<[Vertex; 3]>`. El rasterizador despacha a wireframe, baricéntrico o scanline. El fragment shader es actualmente passthrough.

## Algoritmos y regla de bordes

Ambos algoritmos usan el centro del píxel `(x + 0.5, y + 0.5)`. La regla es inclusiva en `x`, `y` y en las aristas horizontales; incluir ambos extremos horizontales evita huecos, mientras que tomar un único intervalo por fila evita duplicados. Los triángulos degenerados se descartan con `DEGENERATE_EPS = 1e-6` y el bounding box se recorta al framebuffer.

El algoritmo baricéntrico calcula tres funciones de arista por cada píxel del bounding box. Scanline calcula las intersecciones de cada fila y recorre el intervalo entre izquierda y derecha. Por ello, baricéntrico tiene costo por píxel del bounding box y scanline concentra el cálculo geométrico por fila. Los triángulos delgados o diagonales pueden hacer que el bounding box visite muchos píxeles descartados. Ambos crean `Fragment` y pueden hacer crecer el `Vec`.

Una regla top-left aplicada incorrectamente puede asignar un borde a ambos triángulos adyacentes y producir duplicados, o excluirlo de ambos y producir huecos. Este laboratorio usa la regla inclusiva común.

## Imágenes por modo

Generadas con yaw `0.6`, pitch `0.3` y zoom `1.0` mediante `--export-modes`:

![Wireframe](images/mode_wireframe.png)

![Baricéntrico](images/mode_barycentric.png)

![Scanline](images/mode_scanline.png)

![Diferencias](images/mode_diff.png)

La exportación imprimió `Sin diferencias`.

| Modo | Fragmentos |
|---|---:|
| Wireframe | 1,262,853 |
| FilledBarycentric | 1,016,767 |
| FilledScanline | 1,016,779 |

Los conteos internos difieren, pero la imagen diff no contiene diferencias.

## Pruebas de equivalencia

`cargo test -- --nocapture` cubre triángulos pequeños y grandes, aristas horizontales y verticales, coordenadas `.5`, casos fuera del viewport, degenerados, un conteo conocido y 2000 triángulos pseudoaleatorios generados con un LCG determinista.

Resultado real:

```text
4 tests passed
Diferencias de borde: 20 de 20
```

Todas las diferencias aleatorias fueron píxeles a menos de `1e-3` de una arista.

## Benchmark

El benchmark mide solamente `rasterize`, excluyendo `set_pixel`, texturas y ventana. Usa una ejecución de calentamiento, 11 mediciones, 20 iteraciones para la nave y 2000 para el conjunto grande.

CPU real registrada: `11th Gen Intel(R) Core(TM) i7-11800H @ 2.30GHz`. El binario indicó `Release: true`.

| Conjunto | Modo | Fragmentos | Mediana por ejecución | Mín | Máx |
|---|---|---:|---:|---:|---:|
| nave | Wireframe | 1,262,853 | 19,278,434 ns | 17,652,347 ns | 20,413,022 ns |
| nave | FilledBarycentric | 1,016,767 | 32,972,877 ns | 31,917,810 ns | 34,521,004 ns |
| nave | FilledScanline | 1,016,779 | 19,540,252 ns | 18,312,406 ns | 19,868,330 ns |
| grandes | Wireframe | 10,534 | 76,127 ns | 75,016 ns | 79,658 ns |

La corrida completa de `grandes` no terminó en este entorno por el costo de las 2000 iteraciones solicitadas. No se inventan las filas faltantes.

## Flamegraph

No se ejecutó `perf` ni se inspeccionó un SVG en este entorno.

```text
TODO(Marcelo): describir qué funciones ocupan más ancho.
- ¿Predominan los cálculos geométricos?
- ¿Predomina la creación de Fragment?
- ¿Predomina el crecimiento del Vec?
```

En Linux, ejecuta:

```bash
cargo install flamegraph
sudo sysctl kernel.perf_event_paranoid=1
cargo flamegraph -o images/flamegraph_barycentric.svg -- --benchmark-triangles barycentric
cargo flamegraph -o images/flamegraph_scanline.svg -- --benchmark-triangles scanline
```

Si `perf` ya tiene permisos suficientes, no hace falta usar `sudo`.

## Estructura

```text
src/
├── main.rs             # Ventana, exportación y pipeline
├── framebuffer.rs      # Imagen, píxeles y presentación
├── fragment.rs         # Fragmentos
├── vertex.rs           # Vértices
├── shaders.rs          # Vertex shader y fragment shader
├── primitive.rs        # Ensamblaje de triángulos
├── rasterizer.rs       # Modos y despacho
├── triangle.rs         # Wireframe, baricéntrico y scanline
├── scene.rs            # Matrices y triángulos de escena
├── benchmark.rs        # Benchmark sin ventana
├── triangle_tests.rs   # Pruebas de equivalencia
├── obj.rs              # Parser manual OBJ
├── render.rs           # Pipeline anterior de exportación
└── line.rs             # Bresenham y primitivas antiguas

assets/                 # Modelos OBJ
images/                 # Capturas y comparativas
```
