# Taller mecánico: mantenimiento preventivo y repuestos

Backend en Rust (Axum) + CouchDB. Paso 1: registro, login y JWT.

```
taller_mecanico/
├── docker-compose.yml
├── .env.example
└── backend/
    ├── Cargo.toml
    ├── Dockerfile
    └── src/
        ├── main.rs     (rutas y arranque)
        ├── config.rs   (variables de entorno)
        ├── error.rs    (errores -> respuestas HTTP)
        ├── couch.rs    (cliente HTTP de CouchDB)
        └── auth.rs     (registro, login, JWT, Argon2)
```

## Puertos

| Servicio                | Puerto en tu PC |
|-------------------------|-----------------|
| CouchDB nativo (el tuyo)| 5984            |
| CouchDB en Docker       | **5985**        |
| API Rust                | **8090**        |

Los datos de ambos CouchDB son independientes.

## Con Docker (no necesitas instalar Rust)

```bash
cp .env.example .env
nano .env            # cambia COUCHDB_PASSWORD y JWT_SECRET
docker compose up -d --build
docker compose logs -f api
```

La primera compilación tarda varios minutos. Debe aparecer
"CouchDB listo, base 'taller' disponible".

Fauxton: http://localhost:5985/_utils

## Sin Docker (usando tu CouchDB nativo)

Necesitas Rust (`curl https://sh.rustup.rs -sSf | sh`).

```bash
cd backend
export COUCHDB_URL=http://localhost:5984
export COUCHDB_USER=admin
export COUCHDB_PASSWORD=abc123
export JWT_SECRET=$(openssl rand -hex 32)
export BIND_ADDR=0.0.0.0:8090
cargo run
```

## Probar

```bash
curl localhost:8090/health

curl -X POST localhost:8090/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"nombre":"Ana","email":"ana@correo.com","password":"clave12345"}'

curl -X POST localhost:8090/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"ana@correo.com","password":"clave12345"}'

# con el token que devuelve el login:
curl localhost:8090/auth/me -H 'Authorization: Bearer EL_TOKEN'
```

## Desde el celular

La app Flutter solo debe hablar con la API: `http://IP_DE_TU_PC:8090`
(en el emulador Android, `http://10.0.2.2:8090`). Ya no necesita llegar a
CouchDB, así que puedes devolver el CouchDB nativo a `127.0.0.1`:

```bash
curl -X PUT http://admin:abc123@localhost:5984/_node/_local/_config/chttpd/bind_address -d '"127.0.0.1"'
sudo systemctl restart couchdb
```

## Notas

- Las contraseñas se guardan con Argon2id.
- El registro público crea usuarios con rol `dueno`; los mecánicos se
  crearán después desde un endpoint protegido.
- CORS está abierto (`permissive`) solo para desarrollo.
