-- =============================================================================
-- ExposureIQ — Base de Datos: exposureiq_db
-- Migración 002: Datos iniciales (semillas para pruebas y configuración)
-- =============================================================================

-- Contraseña por defecto para usuarios iniciales: "Admin1234!" (argon2id o bcrypt hash demo)
-- Hash bcrypt demo para "Admin1234!"
INSERT INTO usuarios (id, nombre, correo, rol, password_hash, activo)
VALUES 
    ('11111111-1111-1111-1111-111111111111', 'Administrador ExposureIQ', 'admin@exposureiq.internal', 'admin', '$2a$12$eX8L6Gq3/XQjA4w34Eeqz.8qXjW1tXQpQ9Zk1A1h1Vf0L6a7e0rGe', true),
    ('22222222-2222-2222-2222-222222222222', 'Director de Operaciones', 'director@aseguradora.com.mx', 'director', '$2a$12$eX8L6Gq3/XQjA4w34Eeqz.8qXjW1tXQpQ9Zk1A1h1Vf0L6a7e0rGe', true),
    ('33333333-3333-3333-3333-333333333333', 'Analista de Seguridad', 'analista@aseguradora.com.mx', 'analista', '$2a$12$eX8L6Gq3/XQjA4w34Eeqz.8qXjW1tXQpQ9Zk1A1h1Vf0L6a7e0rGe', true)
ON CONFLICT (correo) DO NOTHING;

-- Entidades de prueba para vigilancia (marcas, dominios, ejecutivos clave, patrones)
INSERT INTO entidades_vigiladas (id, tipo, valor, descripcion, activo)
VALUES
    ('44444444-4444-4444-4444-444444444444', 'marca', 'Seguros Alianza', 'Nombre comercial principal de la aseguradora', true),
    ('55555555-5555-5555-5555-555555555555', 'dominio', 'alianzaseguros.com.mx', 'Dominio corporativo de correo y portales web', true),
    ('66666666-6666-6666-6666-666666666666', 'ejecutivo', 'Carlos Mendoza Silva', 'Director General de Operaciones', true),
    ('77777777-7777-7777-7777-777777777777', 'patron_poliza', 'POL-[0-9]{8}-[A-Z]{2}', 'Formato estándar de números de póliza de auto y vida', true)
ON CONFLICT DO NOTHING;
