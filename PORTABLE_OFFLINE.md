# Lightmatter portable sin internet

Hay dos maneras de usar Lightmatter sin conexión:

1. Ejecutar la versión web local desde un navegador que apunte a `http://127.0.0.1:4321`
2. Abrir la carpeta portable y lanzar `start-lightmatter.bat` (Windows) para abrir la app sin depender de internet

## Qué incluye la versión portable

- `lightmatter.exe`
- `public/` con los estilos y recursos estáticos
- `src/pages/` para que la app pueda registrar la navegación sin depender del repositorio original
- `start-lightmatter.bat` para abrir la app y el navegador local

## Cómo usarla

- Descarga el artefacto generado por la acción `Portable Offline Release`
- Extrae el ZIP en cualquier carpeta local
- Haz doble clic en `start-lightmatter.bat`
- La app abre en el navegador local y corre totalmente sin red

## Nota importante

La versión portable no necesita internet para hacer el análisis del laboratorio, pero sí necesita un navegador local (Chrome/Edge/Firefox) para mostrar la interfaz.

Para generar el paquete desde la línea de comandos:

```bash
bash scripts/package-portable.sh
```

En un runner de GitHub Actions, la acción manual `Portable Offline Release` crea el ZIP automáticamente.
