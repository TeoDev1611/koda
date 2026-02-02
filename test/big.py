import sqlite3
import random
from datetime import datetime, timedelta

def generate_large_db(db_name="viewer_pro.db"):
    conn = sqlite3.connect(db_name)
    cursor = conn.cursor()

    # 1. Crear tabla de transacciones masiva
    cursor.execute("""
    CREATE TABLE IF NOT EXISTS transacciones (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        codigo_rastreo TEXT UNIQUE,
        monto REAL NOT NULL,
        tipo TEXT CHECK(tipo IN ('Ingreso', 'Egreso', 'Transferencia')),
        estado TEXT CHECK(estado IN ('Completado', 'Pendiente', 'Fallido')),
        fecha DATETIME,
        notas TEXT
    )
    """)

    # 2. Generar datos ficticios
    tipos = ['Ingreso', 'Egreso', 'Transferencia']
    estados = ['Completado', 'Pendiente', 'Fallido']
    nombres_app = ['Pago suscripción', 'Depósito nómina', 'Compra Amazon', 'Transferencia interna', 'Reembolso']

    print("Generando 1000 registros...")
    
    datos = []
    fecha_base = datetime.now()

    for i in range(1000):
        codigo = f"TRX-{random.randint(100000, 999999)}-{i}"
        monto = round(random.uniform(5.0, 2500.0), 2)
        tipo = random.choice(tipos)
        estado = random.choice(estados)
        # Generar fechas aleatorias en los últimos 30 días
        fecha = fecha_base - timedelta(days=random.randint(0, 30), minutes=random.randint(0, 1440))
        nota = f"{random.choice(nombres_app)} #" + str(random.randint(1, 50))
        
        datos.append((codigo, monto, tipo, estado, fecha.strftime('%Y-%m-%d %H:%M:%S'), nota))

    # 3. Inserción masiva (mucho más rápido que insertar uno por uno)
    cursor.executemany("""
        INSERT INTO transacciones (codigo_rastreo, monto, tipo, estado, fecha, notas) 
        VALUES (?, ?, ?, ?, ?, ?)
    """, datos)

    conn.commit()
    conn.close()
    print(f"✅ ¡Listo! Se han insertado 1000 filas en la tabla 'transacciones'.")

if __name__ == "__main__":
    generate_large_db()