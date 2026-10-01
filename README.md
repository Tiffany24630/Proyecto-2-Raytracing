# Temple of Space — Echoes of Memory

Demo interactiva de raytracing en Rust. El jugador entra por un dominio y visita tres memorias: Desert Pavilion, Mahavaipulya Chamber y Luyang Academy.

## Ejecutar

```powershell
cargo run --release
```

## Controles

| Acción | Teclado y ratón | Mando Xbox |
|---|---|---|
| Interactuar | Clic izquierdo o `E` | `A` |
| Volver desde una sala | `B` | `B` |
| Mover la cámara | `WASD`, flechas o botón derecho | Stick derecho |
| Zoom | Rueda, `+` o `-` | — |
| Activar el Ojo de Dios | `Tab` | `Y` |
| Rotar sello, pintura o pieza | `R` | `X` |
| Pausar animaciones | `P` | `Start` |
| Mover una pieza del puzzle | Flechas y `PgUp`/`PgDn` | D-pad o stick izquierdo |
| Abrir ayuda | `H` | — |
| Repetir diálogo | `N` | — |

El mando se detecta automáticamente. Con mando, `A` interactúa con el objeto que esté al centro de la vista.

## Recorrido

1. Haz clic en la puerta del dominio o pulsa `E` para entrar.
2. En el Santuario, pulsa `Tab` o `Y` para activar el Ojo de Dios.
3. Haz clic en una miniatura para entrar a su memoria.
4. En el desierto, alinea y activa el sello. Cuando Melanta aparezca, no muevas la cámara.
5. En la biblioteca, recoge tres páginas antes de que termine el tiempo.
6. En la academia, selecciona fragmentos, cambia su orden y pulsa el botón central.

## Audio

Coloca los archivos en `assets/audio/`. Los formatos `.mp3`, `.mpeg` y `.ogg` se aceptan; los archivos actuales usan `.mpeg` con contenido MP3.

| Archivo | Evento |
|---|---|
| `music_world.mpeg` | Exterior y Santuario |
| `music_desert.mpeg` | Desert Pavilion |
| `music_library.mpeg` | Mahavaipulya Chamber |
| `music_academy.mpeg` | Luyang Academy |
| `melanta_appear.mpeg` | Aparición de Melanta |
| `enter_miniature.mpeg` | Pantalla de carga al entrar o salir de miniaturas |
| `door_open.mpeg` | Puerta del dominio |
| `page_collect.mpeg` | Recoger página |
| `painting_move.mpeg` | Reordenar pintura |
| `desert_life_lost.mpeg` | Perder una vida en el desierto |
| `task_complete.mp3` | Completar una memoria |

Durante Melanta, la música baja para dar prioridad a su sonido. Puedes definir `TEMPLE_OF_SPACE_MUSIC` para usar un archivo externo como música del mundo.

## Rendimiento

La imagen interactiva se renderiza a `576 × 324` y se escala a una ventana redimensionable. Durante movimiento se usa una previsualización ligera; al detener la cámara se recupera la imagen refinada. El exterior y el Santuario se construyen como vistas separadas para reducir geometría innecesaria.