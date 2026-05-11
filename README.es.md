<div align="center">

# node-token

<p align="center">
  <a href="./README.md">English</a> |
  <a href="./README.zh-CN.md">简体中文</a> |
  <a href="./README.zh-TW.md">繁體中文</a> |
  <a href="./README.es.md">Español</a> |
  <a href="./README.ar.md">العربية</a>
</p>

**Cliente de nodo para PC personal de KeyCompute — trae tu propio cómputo**

<p align="center">
  <a href="https://github.com/keycompute/node-token/stargazers"><img src="https://img.shields.io/github/stars/keycompute/node-token?style=social" alt="GitHub Stars" /></a>
  <a href="https://github.com/keycompute/node-token/issues"><img src="https://img.shields.io/github/issues/keycompute/node-token" alt="GitHub Issues" /></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/License-GPLv3-blue.svg" alt="GPLv3 License" /></a>
  <a href="./CONTRIBUTING.md"><img src="https://img.shields.io/badge/PRs-welcome-brightgreen" alt="PRs Welcome" /></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.92%2B-orange?logo=rust" alt="Rust Version" /></a>
</p>

<p align="center">
  <a href="#características">Características</a> •
  <a href="#inicio-rápido">Inicio rápido</a> •
  <a href="#configuración">Configuración</a> •
  <a href="#uso">Uso</a>
</p>

</div>

---

## Descripción general

`node-token` es un cliente Rust ligero que se ejecuta en PCs personales y las conecta a la plataforma [KeyCompute](https://github.com/keycompute/keycompute) como nodos de cómputo. Sondea el servidor en busca de tareas, las ejecuta en una instancia local de Ollama y envía los resultados — todo sin necesidad de una IP pública.

---

## Características

- **Sondeo pull-based**: funciona detrás de NAT y redes domésticas sin IP pública
- **Ejecución local con Ollama**: ejecuta modelos alojados en Ollama directamente en tu hardware
- **Recuperación automática**: persiste el estado de la sesión localmente y se reanuda tras reinicios
- **Heartbeat de mantenimiento**: heartbeats periódicos mantienen la disponibilidad del nodo
- **Apagado graceful**: deja de aceptar nuevas tareas al salir mientras completa el trabajo en curso
- **Manejo de exclusión**: refleja el estado de exclusión del servidor y continúa con heartbeat de baja frecuencia para visibilidad administrativa

---

## Requisitos previos

| Componente | Versión |
|:---|:---|
| Rust | ≥ 1.92 |
| Ollama | Última |

> Necesitas una instancia de Ollama en ejecución con al menos un modelo descargado. El cliente escanea los modelos locales al iniciar y los reporta durante el registro.

---

## Inicio rápido

### Instalar Ollama

```bash
# Linux
curl -fsSL https://ollama.com/install.sh | sh

# Descargar un modelo
ollama pull gemma3:270m
```

### Compilar y ejecutar node-token

```bash
# Clonar y compilar
git clone https://github.com/keycompute/node-token.git
cd node-token
cp config.example.toml config.toml
# Edita config.toml con la URL de tu servidor KeyCompute y el token de registro

# Compilar
cargo build --release

# Ejecutar
./target/release/node-token
```

### Docker

Usando `docker-compose.yml` (recomendado, incluye Ollama y precalentamiento del modelo):

```bash
# Crear .env desde la plantilla (editar NODE_TOKEN__REGISTRATION_TOKEN)
cp .env.example .env

# Iniciar Ollama + node-token
docker compose up -d

# Ver registros en tiempo real
docker compose logs -f
```

Ejecutar node-token standalone (requiere una instancia de Ollama en ejecución):

```bash
# Construir la imagen
docker build -t node-token .

# Crear volumen de datos
docker volume create node_token_data

# Ejecutar (usar --network host para alcanzar Ollama en el host)
docker run -d \
  --name node-token \
  --network host \
  -v node_token_data:/data \
  -e NODE_TOKEN__SERVER_URL="http://keycompute-server:3000" \
  -e NODE_TOKEN__REGISTRATION_TOKEN="tu-token-de-registro" \
  -e NODE_TOKEN__DISPLAY_NAME="Mi PC Nodo" \
  -e NODE_TOKEN__OLLAMA_URL="http://localhost:11434" \
  node-token
```

---

## Configuración

La configuración se carga desde `config.toml` (o una ruta establecida mediante la variable de entorno `NODE_TOKEN_CONFIG`). Las variables de entorno con el prefijo `NODE_TOKEN__` sobrescriben los valores del archivo.

| Variable | Descripción | Por defecto | Obligatoria |
|:---|:---|:---|:---:|
| `server_url` | URL del servidor KeyCompute | `http://localhost:3000` | ✅ |
| `registration_token` | Token de registro de KeyCompute | — | ✅ |
| `display_name` | Nombre legible del nodo | — | ✅ |
| `ollama_url` | Endpoint de la API local de Ollama | `http://localhost:11434` | ⚪ |
| `heartbeat_interval_secs` | Intervalo de heartbeat en segundos | `30` | ⚪ |
| `excluded_poll_check_interval_secs` | Intervalo de verificación de sondeo cuando está excluido | `30` | ⚪ |
| `data_dir` | Directorio de datos local para persistencia de sesión | `~/.local/share/node-token` | ⚪ |

**Mapeo de variables de entorno**: `NODE_TOKEN__SERVER_URL`, `NODE_TOKEN__REGISTRATION_TOKEN`, etc.

> El `registration_token` y `session_token` nunca se registran en texto plano.

---

## Uso

Una vez que `node-token` está registrado y en ejecución, los usuarios pueden enviar solicitudes a través de la API de KeyCompute usando el prefijo de modelo `node:`:

```bash
curl -s http://tu-servidor-keycompute:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-xxx" \
  -d '{
    "model": "node:gemma3:270m",
    "messages": [{"role": "user", "content": "¡Hola!"}],
    "stream": false
  }'
```

- `node:<modelo>` enruta la solicitud al pool de nodos (solo sin streaming)
- `<modelo>` (sin prefijo) enruta a la ruta normal de cuentas de provider

---

## Cómo funciona

```text
┌─────────────┐     sondear tareas     ┌──────────────────┐
│  node-token │ ◄──────────────────── │  KeyCompute       │
│  (tu PC)    │ ────────────────────► │  Servidor         │
│             │   heartbeat/completar  │                   │
│     │       │                        │        │          │
│     │ llama │                        │        │ encolar  │
│     ▼       │                        │        ▼          │
│  ┌───────┐  │                        │  ┌──────────┐    │
│  │Ollama │  │                        │  │ API de   │    │
│  │       │  │                        │  │ usuario  │    │
│  └───────┘  │                        │  └──────────┘    │
└─────────────┘                        └──────────────────┘
```

1. `node-token` se registra con el servidor KeyCompute, reportando los modelos Ollama disponibles
2. Envía heartbeats periódicos para mantener la sesión activa
3. Realiza sondeo largo de tareas que coincidan con sus modelos aceptados
4. Al recibir una tarea, llama a la instancia local de Ollama y envía el resultado
5. Si es excluido por el servidor (ej. demasiados fallos), deja de sondear pero continúa con heartbeat de baja frecuencia

---

## Desarrollo

```bash
# Compilar
cargo build --release

# Ejecutar pruebas
cargo test --lib
cargo test --tests

# Verificaciones de código
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
```

---

## Estructura del proyecto

```text
node-token/
├── src/
│   ├── main.rs              # Punto de entrada, manejo de señales
│   ├── config.rs            # Gestión de configuración
│   ├── error.rs             # Tipos de error
│   ├── lib.rs               # Raíz de la biblioteca
│   ├── client/              # Clientes HTTP
│   │   ├── api.rs           # Cliente de API KeyCompute
│   │   └── ollama.rs        # Cliente HTTP de Ollama
│   ├── protocol/            # Tipos de protocolo (copiados de keycompute-types)
│   │   ├── types.rs         # DTOs del protocolo de nodo
│   │   └── ollama.rs        # Tipos de API de Ollama
│   ├── runtime/             # Lógica central de tiempo de ejecución
│   │   ├── register.rs      # Lógica de registro
│   │   ├── heartbeat.rs     # Bucle de heartbeat
│   │   ├── poll.rs          # Bucle de sondeo
│   │   └── executor.rs      # Ejecutor de tareas
│   └── storage/             # Persistencia local
│       └── mod.rs           # Almacenamiento de sesión
├── tests/                   # Pruebas de integración
├── benches/                 # Benchmarks
├── config.example.toml
├── .env.example
└── Cargo.toml
```

---

## Licencia

Este proyecto se distribuye bajo la licencia [GNU GPLv3](LICENSE).

---

<div align="center">

### 💖 Gracias por usar node-token

Si este proyecto te ayuda, no dudes en darle una ⭐️ estrella.

**[Inicio rápido](#inicio-rápido)** • **[Reportar problemas](https://github.com/keycompute/node-token/issues)** • **[Últimos lanzamientos](https://github.com/keycompute/node-token/releases)**

</div>
