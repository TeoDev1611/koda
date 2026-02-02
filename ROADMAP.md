# 🗺️ Koda Project Roadmap

Este documento rastrea el desarrollo de Koda, desde un CLI básico hasta una herramienta de base de datos GUI profesional.

## 🟢 Fase 1: CLI & Core (Completado)
- [x] Conexión básica a SQLite, Postgres, MySQL.
- [x] Abstracción `KodaDb` (sqlx).
- [x] Salida en formato tabla en terminal.
- [x] Comandos básicos: `ls`, `query`.

## 🟡 Fase 2: GUI Prototipo (En Progreso)
- [x] Configuración de `egui` + `eframe`.
- [x] Navegador de tablas lateral (árbol jerárquico).
- [x] Editor SQL con resaltado de sintaxis (Keywords y Números).
- [x] Grilla de resultados (Data Grid) con colores inteligentes por tipo de dato.
- [x] Generador de SQL seguro (Update/Delete con previsualización).
- [x] Paridad de funcionalidades con TUI:
    - [x] CRUD básico.
    - [ ] Visualización de BLOBs.
    - [ ] Paginación eficiente.

## 🟠 Fase 3: Funcionalidades Avanzadas (Próximamente)
- [ ] **Schema Introspection Detallado:**
    - Ver columnas, tipos de datos y claves foráneas en el árbol lateral.
- [ ] **Visualización ERD (Diagramas):**
    - Dibujar nodos y conexiones visuales entre tablas.
- [ ] **Editor de Celdas Inline:**
    - Poder editar una celda haciendo doble clic en la grilla (sin generar SQL manual).
- [ ] **Gestión de Conexiones:**
    - Guardar perfiles de conexión en disco (encriptados).

## 🔴 Fase 4: Distribución
- [ ] Empaquetado `.deb`, `.rpm`, `.msi`.
- [ ] Soporte oficial Cross-Platform.