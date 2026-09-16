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
- Ojo de Dios que revela y habilita tres memorias jugables mediante ray casting.
- Tres desafíos breves: quietud en el desierto, páginas contrarreloj y pintura ordenable.
- Timeline de memoria, evento de Melanta y skybox procedural.
- Interfaz y narrativa integradas en el framebuffer.

## Ejecutar

Usa el perfil optimizado:

```bash
cargo run --release
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

### Ojo de Dios

El Ojo de Dios funciona como modo de percepción. Al activarlo en el templo, las miniaturas de Luyang, Mahavaipulya y Desert Pavilion se convierten en accesos a sus memorias preservadas. No existe un puzzle obligatorio antes de entrar a las salas.

El raycast solo considera geometría interactiva y admite un pequeño margen alrededor del cursor para evitar que columnas u otros elementos decorativos bloqueen la selección.

| Acción | Tecla |
|---|---|
| Activar el modo en el templo | `Tab` |
| Apuntar | Ratón |
| Entrar a una miniatura | Clic izquierdo o `E` |
| Regresar al templo desde una sala | `B` |

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
3. Apunta a una miniatura y usa clic izquierdo o `E` para entrar.
4. En Desert Pavilion, gira y confirma el sello; cuando aparezca Melanta, permanece inmóvil.
5. En Mahavaipulya, recoge las tres páginas antes de que termine el cronómetro.
6. En Luyang, selecciona una sección, muévela con `R` y confirma el orden en el botón central. La solución inicial requiere seleccionar `A` y pulsar `R` dos veces.
7. Usa `B` para volver al templo y continúa con las demás memorias.
8. Al regresar después de completar las tres salas, escucha el mensaje final de Nihilita. Pulsa `E` para terminar.

## Rendimiento

La resolución interactiva es `576 × 324`, mostrada inicialmente a escala doble en una ventana redimensionable de `1152 × 648`. El renderer:

- recalcula únicamente cuando cambia la cámara, la escena o una animación;
- usa una previsualización `400 × 225` con filtrado bilineal durante el movimiento; conserva rayos primarios, intersecciones, texturas, iluminación y sombras, y recupera reflexión/refracción con resolución completa tras 120 ms de reposo real;
- pospone el refinado mientras el botón derecho o una tecla de cámara sigan pulsados, evitando que un render completo bloquee controles a mitad de un gesto;
- mantiene dos niveles de rayos secundarios en la ventana interactiva y reserva los cuatro niveles de `MAX_DEPTH` para las capturas de `--render-once`;
- limita los saltos máximos del ratón y calcula la órbita de teclado según el tiempo transcurrido para evitar movimientos bruscos cuando un render tarda más de lo normal;
- dibuja la interfaz después del escalado para mantener el texto nítido;
- utiliza como máximo ocho workers;
- termina cada consulta de sombra al encontrar el primer obstáculo, sin alterar la imagen;
- descarta rayos con aporte final igual o menor al 5%;
- carga las texturas en RGB8 una sola vez;
- limita la órbita para evitar atravesar paredes;
- conserva el bucle de interfaz a 30 Hz cuando no hay un render activo.