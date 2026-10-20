# Lab 5: Rasterización de triángulos

Implementación de un renderer por software desarrollado desde cero en Rust con `raylib`. El proyecto carga modelos Wavefront `.obj`, procesa sus vértices mediante uniforms y shaders, ensambla triángulos y los rasteriza como fragmentos sobre un framebuffer propio.

Este Laboratorio 5 es la continuación del trabajo desarrollado en el Laboratorio 4. El repositorio de la etapa anterior puede consultarse aquí: [Lab 4: lab4-graficas](https://github.com/MarceloDetlefsen/lab4-graficas.git).

![Nave renderizada](images/mode_wireframe.png)

## Cómo correr el proyecto

### Requisitos

- **Rust + Cargo** (edición 2024 o versión moderna instalada).
- Herramientas de compilación de C y CMake para `raylib-sys`.

### Build y ejecución

**Abrir el renderer interactivo:**

```bash
cargo run --release
```

**Exportar un modelo individual:**

```bash
cargo run --release -- assets/Nave_X_Y.obj images/salida.png
```

**Exportar los tres modos y la imagen de diferencias:**

```bash
cargo run --release -- --export-modes
```

**Ejecutar el benchmark sin abrir ventana:**

```bash
cargo run --release -- --benchmark-triangles
cargo run --release -- --benchmark-triangles barycentric
cargo run --release -- --benchmark-triangles scanline
```

**Ejecutar las pruebas:**

```bash
cargo test
cargo test -- --nocapture
```

La ventana utiliza un framebuffer fijo de `800x600`. Sus controles son:

- Flechas izquierda/derecha: rotación horizontal.
- Flechas arriba/abajo: rotación vertical.
- Rueda del mouse: zoom.
- `1` y `2`: rotación automática horizontal y vertical.
- `R`: reiniciar cámara, zoom y movimiento automático.
- `W`, `B` y `S`: wireframe, relleno baricéntrico y relleno scanline.

## Descripción

El proyecto implementa un pipeline gráfico completo por software, sin utilizar las funciones 3D automáticas de Raylib:

- **Parser manual OBJ:** carga vértices (`v`) y caras (`f`), convirtiendo índices OBJ base 1 a índices Rust base 0.
- **Uniforms y matriz de modelo:** todos los vértices de un dibujo comparten la misma matriz.
- **Vertex shader:** transforma cada vértice usando una posición homogénea `Vector4`.
- **Primitive assembly:** agrupa vértices consecutivos en triángulos de tres elementos.
- **Rasterización:** genera fragmentos mediante wireframe, coordenadas baricéntricas o scanline.
- **Fragment shader:** devuelve el color del fragmento sin modificarlo.
- **Framebuffer propio:** escribe píxeles en una imagen de `800x600` y la presenta en la ventana o la exporta a PNG.

### Preparación del modelo

El modelo principal es `assets/Nave_X_Y.obj`, exportado como malla triangulada. También se conserva `assets/cube.obj` como modelo pequeño de prueba.

---

## Pipeline de Renderizado

```text
Archivo .OBJ
    ↓
Parser manual (load_obj): vértices e índices
    ↓
Vertex shader: aplicación de uniforms.model_matrix
    ↓
Primitive assembly: grupos de tres vértices
    ↓
Rasterización: wireframe, baricéntrico o scanline
    ↓
Fragment shader: selección del color
    ↓
Framebuffer::set_pixel
    ↓
Ventana o exportación PNG
```

### Regla común de rasterización

El centro del píxel `(x, y)` se evalúa en `(x + 0.5, y + 0.5)`. La regla de borde es inclusiva: los centros sobre aristas y vértices pertenecen al triángulo. Las aristas horizontales incluyen ambos extremos para evitar huecos; el intervalo por fila evita duplicados. Los triángulos degenerados se descartan con `DEGENERATE_EPS = 1e-6` y los bounding boxes se recortan al framebuffer.

Una regla top-left aplicada de forma distinta puede causar duplicados entre triángulos adyacentes si ambos incluyen el borde, o huecos si ambos lo excluyen. En este laboratorio ambos algoritmos comparten la regla inclusiva.

### Modos de rasterización

El modo **baricéntrico** calcula tres funciones de arista para cada centro de píxel del bounding box. El modo **scanline** calcula las intersecciones de cada fila horizontal y rellena entre la izquierda y la derecha. Wireframe reutiliza la implementación de aristas del pipeline anterior, pero en esta etapa genera fragmentos mediante interpolación lineal tipo DDA: usa `ceil(max(|dx|, |dy|))` pasos. Bresenham permanece en `line.rs` como parte del pipeline anterior de exportación.

## Comparativa de resultados

Las imágenes se generan con:

```bash
cargo run --release -- --export-modes
```

La exportación utiliza yaw `0.6`, pitch `0.3`, zoom `1.0` y el color naranja pálido del renderer.

| Modo | Resultado |
| :---: | :---: |
| Wireframe | ![Wireframe](images/mode_wireframe.png) |
| Baricéntrico | ![Baricéntrico](images/mode_barycentric.png) |
| Scanline | ![Scanline](images/mode_scanline.png) |
| Diferencias | ![Diferencias](images/mode_diff.png) |

`mode_diff.png` es una máscara: pinta de rojo solo los píxeles distintos. Por eso aparece con el fondo cuando la salida imprime `Sin diferencias`; no es una imagen adicional del modelo.

## Pruebas de equivalencia

Las pruebas cubren triángulos pequeños y grandes, aristas horizontales y verticales, coordenadas enteras y `.5`, triángulos parcialmente o completamente fuera del framebuffer, degenerados, un conteo conocido y 2000 triángulos pseudoaleatorios generados con un LCG determinista.

Resultado real:

```text
5 tests passed
Diferencias de borde: 20 de 20
```

Las 20 diferencias encontradas por la prueba aleatoria estuvieron a menos de `1e-3` de una arista.

### Diferencia de fragmentos entre rellenos

El diagnóstico con la nave, usando yaw `0.6`, pitch `0.3` y zoom `1.0`, reportó:

```text
Triángulos distintos: 12
Píxeles distintos acumulados: 12
Solo scanline: 12
Solo baricéntrico: 0
Distancia máxima a arista: 0.00001526
Píxeles distintos en framebuffer final: 0
```

La hipótesis se confirma: scanline interpola la intersección `x` sobre cada arista y el algoritmo baricéntrico evalúa funciones de arista. El redondeo en `f32` puede decidir de manera distinta únicamente para centros casi exactamente sobre una arista. Scanline produce los 12 píxeles adicionales: el diagnóstico reporta `solo scanline = 12` y `solo baricéntrico = 0`. `mode_diff.png` compara el framebuffer final, no la lista de fragmentos; el overdraw de triángulos posteriores oculta esas 12 diferencias.

## Benchmark

El benchmark mide únicamente la generación de fragmentos en `rasterize`. Excluye `set_pixel`, texturas, ventana y esperas. Usa una ejecución de calentamiento y 11 mediciones. La nave usa 20 iteraciones, `grandes` usa 100 y `delgados` usa 200; estas últimas hacen que cada medición dure cientos de milisegundos y reduzca el ruido.

CPU registrada: **11th Gen Intel(R) Core(TM) i7-11800H @ 2.30GHz**. El binario indicó `Release: true`.

| Conjunto | Modo | Fragmentos | Mediana por ejecución | Mín | Máx |
|---|---|---:|---:|---:|---:|
| nave | Wireframe | 1,262,853 | 17,219,852 ns | 16,920,950 ns | 18,034,448 ns |
| nave | FilledBarycentric | 1,016,767 | 32,735,862 ns | 32,449,746 ns | 33,440,353 ns |
| nave | FilledScanline | 1,016,779 | 19,465,803 ns | 19,193,217 ns | 20,040,093 ns |
| grandes | Wireframe | 10,534 | 73,431 ns | 72,158 ns | 77,198 ns |
| grandes | FilledBarycentric | 704,044 | 7,274,664 ns | 7,143,826 ns | 7,371,831 ns |
| grandes | FilledScanline | 704,044 | 2,157,334 ns | 2,119,439 ns | 2,214,000 ns |
| delgados | Wireframe | 12,536 | 89,049 ns | 87,619 ns | 89,626 ns |
| delgados | FilledBarycentric | 11,322 | 3,490,170 ns | 3,464,877 ns | 3,574,333 ns |
| delgados | FilledScanline | 11,322 | 100,445 ns | 99,004 ns | 113,272 ns |

En los tres conjuntos medidos, scanline fue más rápido que baricéntrico: aproximadamente 1.68 veces en la nave, 3.37 veces en `grandes` y 34.75 veces en `delgados`, según las medianas anteriores. El conjunto `delgados` confirma que un bounding box grande respecto al área cubierta penaliza fuertemente al algoritmo baricéntrico; scanline evita visitar la mayoría de esos píxeles y trabaja principalmente por fila. Ambos algoritmos generan `Fragment` y hacen crecer el `Vec`.

## Flamegraph

Los flamegraphs fueron generados en Arch Linux con `perf` y el binario release con símbolos de depuración:

```bash
cargo install flamegraph
sudo pacman -S perf
sudo sysctl kernel.perf_event_paranoid=1
cargo flamegraph -o images/flamegraph_barycentric.svg -- --benchmark-triangles barycentric
cargo flamegraph -o images/flamegraph_scanline.svg -- --benchmark-triangles scanline
```

Si `perf` ya tiene permisos suficientes, no hace falta utilizar `sudo`.

![Flamegraph baricéntrico](images/flamegraph_barycentric.svg)

![Flamegraph scanline](images/flamegraph_scanline.svg)

En el flamegraph baricéntrico, `triangle_barycentric` representa aproximadamente el **50.04%** de las muestras. También aparecen `rasterize` con **22.81%**, `append_elements<Fragment>` con **22.76%**, `push<Fragment>` con **14.71%** y `write<Fragment>` con **10.52%**. `asm_exc_page_fault` aparece en ramas asociadas a la reserva y primer acceso de memoria con valores de aproximadamente **18.63%** y **17.74%**; no se suman porque son frames repetidos de stacks distintos. Esto coincide con el costo de tocar páginas nuevas de `Vec` grandes, incluyendo el `with_capacity` por triángulo y el `extend` al vector global.

En scanline, la inspección de coordenadas del SVG confirma que `push<Fragment>` (**35.66%**) está contenido horizontalmente dentro de `triangle_scanline` (**65.09%**), mientras `append_elements<Fragment>` (**29.24%**) está contenido dentro de `rasterize`, no dentro de `triangle_scanline`. Por tanto, la parte geométrica propia aproximada de scanline es `65.09% - 35.66% = 29.43%`, y el almacenamiento de fragmentos (`push` + `append_elements`) ronda `64.90%`. El SVG también muestra `realloc_nonnull` con **0.80%** y `asm_exc_page_fault` con valores de hasta **1.32%**; no se suman frames repetidos.

En baricéntrico, aunque `triangle_barycentric` ocupa **50.04%**, sus hijos aparecen en ramas separadas y los frames de `push`, `with_capacity` y fallos de página no permiten determinar con claridad una resta equivalente sin sumar stacks repetidos. Por eso no se presenta una cifra de costo geométrico propio para ese algoritmo. La lectura debe hacerse con cuidado: los porcentajes del flamegraph son muestras de los stacks y algunas funciones aparecen en varias ramas. Cada triángulo devuelve su propio `Vec<Fragment>` y luego ese vector se copia al vector global mediante `extend`, por lo que existe un costo doble de almacenamiento. No se elimina porque el enunciado exige que las funciones `triangle_*` devuelvan `Vec<Fragment>`.

## Estructura del proyecto

```text
.
├── .gitignore          # Archivos y directorios ignorados por Git
├── README.md           # Documentación del proyecto, pipeline y comparativas
├── Cargo.toml          # Configuración del crate, perfiles y dependencias
├── Cargo.lock          # Versiones bloqueadas de dependencias
├── assets/
│   ├── Nave_X_Y.obj    # Modelo principal de la nave
│   └── cube.obj        # Modelo 3D básico de prueba
├── images/
│   ├── mode_wireframe.png         # Render wireframe
│   ├── mode_barycentric.png       # Relleno baricéntrico
│   ├── mode_scanline.png          # Relleno scanline
│   ├── mode_diff.png              # Diferencias entre rellenos
│   ├── flamegraph_barycentric.svg # Perfil del algoritmo baricéntrico
│   └── flamegraph_scanline.svg    # Perfil del algoritmo scanline
└── src/
    ├── main.rs         # Punto de entrada, ventana y pipeline
    ├── framebuffer.rs  # Búfer de píxeles, set_pixel y exportación
    ├── fragment.rs     # Estructura Fragment
    ├── vertex.rs       # Estructura Vertex
    ├── shaders.rs      # Vertex shader y fragment shader
    ├── primitive.rs    # Ensamblaje de triángulos
    ├── rasterizer.rs   # Modos y despacho de rasterización
    ├── triangle.rs     # Wireframe, baricéntrico y scanline
    ├── scene.rs        # Matrices y construcción de triángulos
    ├── benchmark.rs    # Benchmark sin ventana
    ├── triangle_tests.rs # Pruebas de equivalencia
    ├── obj.rs          # Parser manual de OBJ
    ├── render.rs       # Pipeline anterior de exportación
    └── line.rs         # Algoritmo de líneas de Bresenham
```

## Limitaciones actuales

- No hay z-buffer ni eliminación de caras ocultas.
- La proyección es ortográfica; todavía no hay perspectiva ni división por `w`.
- El color y la profundidad de los fragmentos rellenos son uniformes en esta etapa.

## Dependencias principales

- [`raylib`](https://crates.io/crates/raylib) — Tipos geométricos, `Image`, ventana y carga de texturas para presentar el framebuffer. No se usa para cargar ni dibujar modelos 3D.

## Autor

**Marcelo Detlefsen** - Carné 24554  
*Universidad del Valle de Guatemala*  
*Gráficas por Computadora - Laboratorio 5*
