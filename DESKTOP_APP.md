# Lightmatter desktop + PWA

Este repositorio ya tiene la base para una app desktop con Tauri y una distribución portable/offline de la app web.

## Objetivo

- generar un exe portátil para Windows
- abrir la app localmente en un navegador web embedded
- funcionar sin internet cuando no haya red
- actualizarse automáticamente cuando haya conexión disponible

## Stack recomendado

- Rust + Resuma para el backend y los workers
- Tauri como shell de escritorio + ventana local
- PWA para instalación web y actualización automática

## Arranque local del escritorio

```bash
cargo install tauri-cli
cargo tauri dev
```

## Build de producción

```bash
cargo tauri build
```

La versión desktop apuntará a `http://127.0.0.1:4321` y intentará arrancar el servidor local de Lightmatter si está disponible.

## Offline

La app usa el almacenamiento local en `data/` dentro del paquete para mantener la cola de Resuma y los trabajos terminados aunque no haya internet.
