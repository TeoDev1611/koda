#[derive(Clone, Copy)]
pub enum Language {
    En,
    Es,
}

pub struct Strings;

impl Strings {
    pub fn get(lang: &Language, key: &str) -> String {
        match (lang, key) {
            // Errors
            (Language::En, "error_foreign_key") => {
                "Cannot delete: Data is referenced by another table.".to_string()
            }
            (Language::Es, "error_foreign_key") => {
                "No se puede borrar: Datos referenciados por otra tabla.".to_string()
            }
            (Language::En, "error_delete_failed") => "Delete failed".to_string(),
            (Language::Es, "error_delete_failed") => "Error al borrar".to_string(),

            // Header / Status
            (Language::En, "status_disconnected") => "Disconnected".to_string(),
            (Language::Es, "status_disconnected") => "Desconectado".to_string(),
            (Language::En, "status_connecting") => "Connecting to".to_string(),
            (Language::Es, "status_connecting") => "Conectando a".to_string(),
            (Language::En, "status_connected") => "Connected".to_string(),
            (Language::Es, "status_connected") => "Conectado".to_string(),
            (Language::En, "status_failed") => "Failed".to_string(),
            (Language::Es, "status_failed") => "Error".to_string(),

            // Titles
            (Language::En, "title_tables") => " Tables (Tab) ".to_string(),
            (Language::Es, "title_tables") => " Tablas (Tab) ".to_string(),
            (Language::En, "title_data") => " Data: ".to_string(),
            (Language::Es, "title_data") => " Datos: ".to_string(),

            // Hints
            (Language::En, "hint_nav") => "Tab: Switch | '?' Help | 'q' Exit".to_string(),
            (Language::Es, "hint_nav") => "Tab: Cambiar Foco | '?' Ayuda | 'q' Salir".to_string(),
            (Language::En, "hint_add") => "'a' to Add".to_string(),
            (Language::Es, "hint_add") => "'a' Añadir".to_string(),
            (Language::En, "hint_select") => "Select a table and press Enter".to_string(),
            (Language::Es, "hint_select") => "Selecciona una tabla y pulsa Enter".to_string(),

            // Actions
            (Language::En, "action_adding") => "Adding Row".to_string(),
            (Language::Es, "action_adding") => "Añadiendo Fila".to_string(),
            (Language::En, "action_editing") => "Editing Row".to_string(),
            (Language::Es, "action_editing") => "Editando Fila".to_string(),
            (Language::En, "original") => "Original".to_string(),
            (Language::Es, "original") => "Original".to_string(),

            // Help
            (Language::En, "help_title") => "Help".to_string(),
            (Language::Es, "help_title") => "Ayuda".to_string(),
            (Language::En, "help_nav_title") => "Navigation:".to_string(),
            (Language::Es, "help_nav_title") => "Navegación:".to_string(),
            (Language::En, "help_nav_desc") => "  Up/Down: Select Table/Row".to_string(),
            (Language::Es, "help_nav_desc") => "  Arr/Abj: Seleccionar Tabla/Fila".to_string(),
            (Language::En, "help_tab") => "  Tab:     Switch Focus".to_string(),
            (Language::Es, "help_tab") => "  Tab:     Cambiar Foco".to_string(),
            (Language::En, "help_enter") => "  Enter:   Load Data / Confirm".to_string(),
            (Language::Es, "help_enter") => "  Enter:   Cargar Datos / Confirmar".to_string(),
            (Language::En, "help_edit_title") => "Editing:".to_string(),
            (Language::Es, "help_edit_title") => "Edición:".to_string(),
            (Language::En, "help_edit_a") => "  a:       Add Row".to_string(),
            (Language::Es, "help_edit_a") => "  a:       Añadir Fila".to_string(),
            (Language::En, "help_edit_e") => "  e:       Edit Row".to_string(),
            (Language::Es, "help_edit_e") => "  e:       Editar Fila".to_string(),
            (Language::En, "help_edit_x") => "  x:       Delete Row".to_string(),
            (Language::Es, "help_edit_x") => "  x:       Borrar Fila".to_string(),
            (Language::En, "help_general") => "General:".to_string(),
            (Language::Es, "help_general") => "General:".to_string(),
            (Language::En, "help_lang") => "  l:       Switch Language".to_string(),
            (Language::Es, "help_lang") => "  l:       Cambiar Idioma".to_string(),

            // Delete
            (Language::En, "confirm_delete_title") => " Confirm Delete ".to_string(),
            (Language::Es, "confirm_delete_title") => " Confirmar Borrado ".to_string(),
            (Language::En, "confirm_delete_msg") => {
                "Are you sure you want to delete this row?".to_string()
            }
            (Language::Es, "confirm_delete_msg") => {
                "¿Seguro que deseas borrar esta fila?".to_string()
            }
            (Language::En, "confirm_yes") => "(y) Yes".to_string(),
            (Language::Es, "confirm_yes") => "(y) Sí".to_string(),
            (Language::En, "confirm_no") => "(n) No".to_string(),
            (Language::Es, "confirm_no") => "(n) No".to_string(),

            _ => key.to_string(),
        }
    }
}
