import sqlite3

def setup_database(db_name="viewer_testing.db"):
    try:
        # Conexión a la base de datos (se crea el archivo si no existe)
        conn = sqlite3.connect(db_name)
        cursor = conn.cursor()

        # Script SQL consolidado
        sql_script = """
        -- Tablas
        CREATE TABLE IF NOT EXISTS usuarios (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL,
            activo BOOLEAN DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS categorias (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS productos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            precio REAL NOT NULL,
            categoria_id INTEGER,
            FOREIGN KEY (categoria_id) REFERENCES categorias(id)
        );

        -- Datos de prueba iniciales
        INSERT OR IGNORE INTO categorias (nombre) VALUES ('Software'), ('Hardware');
        
        INSERT OR IGNORE INTO usuarios (nombre, email) VALUES 
        ('Admin User', 'admin@example.com'),
        ('Dev User', 'dev@example.com');

        INSERT OR IGNORE INTO productos (nombre, precio, categoria_id) VALUES 
        ('Visual Studio Code', 0.0, 1),
        ('Monitor Ultrawide', 450.99, 2);
        """

        # Ejecutar script completo
        cursor.executescript(sql_script)
        
        # Guardar cambios
        conn.commit()
        print(f"✅ Base de datos '{db_name}' creada y poblada con éxito.")

    except sqlite3.Error as e:
        print(f"❌ Error al crear la base de datos: {e}")
    
    finally:
        if conn:
            conn.close()

if __name__ == "__main__":
    setup_database()