# --- ETAPA 1: BUILDER (El obrero) ---
# Usamos la imagen oficial de Rust para compilar.
# Le ponemos un alias "builder" para referenciarla después.
FROM rust:1.75 as builder

# Creamos un directorio de trabajo dentro del contenedor
WORKDIR /usr/src/app

# Copiamos todo el contenido de tu carpeta actual al contenedor
COPY . .

# Compilamos el proyecto en modo release.
# El flag --release es clave porque optimiza el binario (lo hace más rápido y liviano).
RUN cargo build --release

# --- ETAPA 2: RUNNER (El producto final) ---
# Usamos una imagen base de Debian liviana (slim).
# Rust linkea dinámicamente con libc, así que necesitamos un SO base mínimo.
# (Si usaras MUSL podrías usar 'scratch' o 'alpine', pero Debian es más compatible para arrancar).
FROM debian:bookworm-slim

# Actualizamos certificados y librerías básicas por seguridad (buena práctica de ciberseguridad)
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*

# Copiamos SOLAMENTE el binario desde la etapa "builder".
# OJO: Reemplazá "nombre_de_tu_app" por el nombre real que está en tu Cargo.toml
COPY --from=builder /usr/src/app/target/release/SO-Fase1 /usr/local/bin/mi-app

RUN mkdir -p /input

# Declaramos que esto es un volumen (informativo para quien lea el Dockerfile)
VOLUME ["/input"]

CMD ["mi-app"]