# Temple of Space — Echoes of Memory

Demo técnica interactiva de raytracing desarrollada en Rust. Presenta un diorama original inspirado en el Templo del Espacio de *Genshin Impact*: un museo celestial suspendido donde distintas culturas y recuerdos permanecen preservados alrededor del Memory Core.

La cámara y las propiedades ópticas forman parte de la experiencia. Reflexión, refracción, materiales, posición y perspectiva intervienen directamente en la reconstrucción de una memoria fragmentada.

## Identidad visual

- Windrest Peak combina pradera verde estilizada, ruinas, rocas y energía de viento.
- Salón central amplio, abierto al cielo y organizado mediante un eje ceremonial.
- Mármol azul-gris, ornamentación de oro antiguo y cristal azul celeste.
- Diez columnas monumentales y plataformas de exhibición separadas.
- Luyang Academy representada mediante madera lacada, pinturas, tinta y pergaminos.
- Mahavaipulya Chamber representada mediante estanterías, libros y páginas suspendidas.
- Desert Pavilion representado mediante arena, pilares, arco y escultura.
- Memory Core central rodeado por una jaula geométrica y sellos luminosos.
- Corrupción de Melanta reservada para el estado rojo final.

La escena no reutiliza modelos ni texturas extraídos del juego. Es una interpretación académica construida con primitivas y recursos originales.

## Características técnicas

- Raytracer recursivo implementado por el proyecto.
- Intersecciones con planos y cubos orientados.
- Sombras e iluminación difusa y especular.
- Reflexión real con límite de profundidad.
- Refracción mediante ley de Snell, Fresnel y reflexión interna total.
- Cinco materiales: piedra, madera, metal, cristal y tinta/energía.
- Texturas PNG estilizadas para los cinco materiales y el terreno, con UV, repetición y filtrado bilineal.
- Cámara orbital limitada al interior del salón.
- Selección de objetos mediante ray casting.
- Puzzle que requiere posición, rotación y perspectiva correctas.
- Timeline de memoria, evento de Melanta y skybox procedural.
- Interfaz y narrativa integradas en el framebuffer.

## Ejecutar

Usa el perfil optimizado:

```bash
cargo run --release
```

Para generar la galería de estados finales:

```bash
cargo run --release -- --render-once
```

## Controles

### Cámara

| Acción | Tecla |
|---|---|
| Rotación horizontal | `A` / `D` o flechas izquierda/derecha |
| Rotación vertical | `W` / `S` o flechas arriba/abajo |
| Órbita libre | Arrastrar con botón derecho del ratón |
| Zoom | Rueda del ratón o `+` / `-` |
| Entrar o regresar por el portal | `E` |

### Ojo de Dios y vínculo espacial de Dex

El Ojo de Dios funciona como modo de percepción. Dex establece un vínculo con el fragmento señalado por el ratón; las siluetas celestes indican los destinos preservados y los marcadores brillantes confirman qué pieza está vinculada.

El mensaje central cambia según el error actual y explica la siguiente acción necesaria: seleccionar, subir, bajar, desplazar, rotar o corregir la cámara.

Las tolerancias aceptan pequeños errores de posición, rotación y cámara. Las piezas no se colocan automáticamente: el jugador continúa realizando la reconstrucción.

| Acción | Tecla |
|---|---|
| Activar o cerrar el modo | `Tab` |
| Apuntar | Ratón |
| Vincular o soltar | Clic izquierdo o `E` |
| Mover en X/Z | Flechas |
| Mover verticalmente en Y | `PageUp` / `PageDown` |
| Rotar el fragmento | `R` |
| Cancelar el movimiento actual | `Esc` |

### Memoria

| Acción | Tecla |
|---|---|
| Reiniciar la timeline | `T` |
| Pausar o continuar | `Espacio` |
| Adelantar la intervención automática de Melanta | `M` |
| Restaurar la memoria tras la corrupción | `F` |

## Recorrido de demostración

1. Pulsa `E` en el exterior para cruzar el portal.
2. Explora el salón con la cámara y pulsa `Tab` para activar el Ojo de Dios.
3. Apunta a un fragmento y pulsa `E` para vincularlo.
4. Llévalo a una silueta celeste, rota la pieza y pulsa `E` para soltarla.
5. Repite el proceso con los tres fragmentos.
6. Ajusta la cámara hasta que el indicador confirme la perspectiva correcta.
7. Observa la reconstrucción; Melanta intervendrá automáticamente después de una breve pausa. `M` permite adelantarla.
8. Cuando la corrupción alcance 100%, pulsa `F`.

### Solución reproducible

```text
Pieza A: Right ×4, PageUp ×3, Up ×1, R ×2
Pieza B: PageDown ×3, Up ×1, R ×3
Pieza C: Left ×4, PageUp ×2, Up ×2, R ×4
Cámara:  D ×3, + ×2
```

Confirma cada pieza con `E` antes de seleccionar la siguiente.

## Rendimiento

La resolución interactiva es `576 × 324`, mostrada inicialmente a escala doble en una ventana redimensionable de `1152 × 648`. El renderer:

- recalcula únicamente cuando cambia la cámara, la escena o una animación;
- usa una previsualización `352 × 198`, filtrado bilineal y un rebote durante el movimiento, y recupera automáticamente la resolución completa con `MAX_DEPTH` tras 60 ms de reposo;
- dibuja la interfaz después del escalado para mantener el texto nítido;
- utiliza como máximo ocho workers;
- descarta rayos con aporte final igual o menor al 5%;
- carga las texturas en RGB8 una sola vez;
- limita la órbita para evitar atravesar paredes;
- conserva el bucle de interfaz a 30 Hz cuando no hay un render activo.

## Validación

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo build --release
```
