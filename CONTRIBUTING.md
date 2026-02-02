# Contributing to Koda

¡Gracias por querer mejorar Koda!

## Requisitos de Desarrollo

### Dependencias de Sistema (Linux)
Para compilar la versión GUI, necesitas instalar las siguientes librerías:
```bash
sudo apt-get install -y libwayland-dev libx11-dev libxkbcommon-dev libxcursor-dev libxrandr-dev libxi-dev libasound2-dev
```

## Flujo de Trabajo
1. Crea una rama para tu característica: `git checkout -b feature/mi-mejora`.
2. Asegúrate de que el código compila: `cargo check --all-targets`.
3. Ejecuta los tests: `cargo test`.
4. Verifica el formato: `cargo fmt --all`.
5. Abre un Pull Request.

## Estructura del Proyecto
- `src/db`: Lógica central de bases de datos.
- `src/gui.rs`: Interfaz Gráfica (egui).
- `src/ui`: Interfaz de Terminal (ratatui).
- `src/main.rs`: Lógica de línea de comandos.
