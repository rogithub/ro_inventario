# ro_inventario

Inventario y punto de venta para negocios chicos en México, en Rust. Qué es y por qué: [`VISION.md`](VISION.md). Cómo se trabaja y las reglas: [`CLAUDE.md`](CLAUDE.md).

Esta página es el recordatorio para **probar en una computadora nueva** (la laptop). Todos los comandos se corren desde la raíz del repo.

---

## Una sola vez

### 1. Programas
- **Rust** con [rustup](https://rustup.rs). La versión exacta la elige `rust-toolchain.toml` sola, la primera vez que se compila.
- **Podman** (para Postgres) y **Node** (para las pruebas E2E).
- Herramientas de cargo (tardan unos minutos):

```sh
cargo install sqlx-cli --version 0.9.0 --no-default-features --features postgres --locked
```

```sh
cargo install cargo-audit --locked
```

### 2. Postgres en un contenedor
Los scripts esperan un contenedor de podman llamado **`postgres`**, con el puerto **5432** publicado en la laptop (CI usa `postgres:18`). `herramientas/dev-db.sh` entra a él como el superusuario `postgres` con `podman exec`. Para ver que está:

```sh
podman ps --filter name=postgres
```

### 3. El secreto de desarrollo
Crear `.secretos/dev.env` (git lo ignora) **con emacs**, nunca con `echo`. Una sola línea, con 16 o más letras o números:

```
DEV_DB_PASSWORD=inventaUnaLargaAqui123
```

Después, que solo tú lo puedas leer:

```sh
chmod 600 .secretos/dev.env
```

### 4. La base de desarrollo
Crea el usuario `ro_inventario` y la base `dev_ro_inventario` desde cero:

```sh
herramientas/dev-db.sh
```

Para consultar la copia de la v1 (`dev_inventario_papeleria`, si la tienes en el mismo Postgres), dale a `ro_inventario` permiso de solo lectura. Se corre otra vez cada vez que se refresca la copia:

```sh
herramientas/dev-v1-lectura.sh
```

### 5. Tu usuario para entrar
Pide la contraseña dos veces sin mostrarla. Al correr, también aplica las migraciones.

```sh
. herramientas/dev-env.sh && cargo run -p privada -- crear-usuario --email tu@correo --nombre "Tu nombre" --rol Dueño
```

### 6. Para las pruebas E2E (opcional)
Los navegadores de Playwright:

```sh
cd e2e && npm ci && npx playwright install firefox chromium && cd ..
```

Las pruebas entran con el usuario de la IA. Sus credenciales van como `E2E_USER` y `E2E_PASS` en tu `~/.zshrc` (editado con emacs). Luego se crea en la base:

```sh
zsh -ic 'herramientas/dev-usuario-ia.sh'
```

---

## Cada vez que quieras probar

### 1. Traer lo último

```sh
git pull
```

### 2. Prender Postgres
Si la laptop se reinició:

```sh
podman start postgres
```

### 3. Correr la aplicación

```sh
. herramientas/dev-env.sh && NEGOCIO_CONFIG=negocio.ejemplo.toml PORT=5100 cargo run -p privada
```

- Abrir <http://localhost:5100> y entrar con tu usuario.
- ¿Está viva? En otra terminal: `curl localhost:5100/health`
- Para verla en el iPad (misma red): `http://<ip-de-la-laptop>:5100`
- Detenerla: `Ctrl+C`.

### 4. Revisar todo, como CI (opcional)
Formato, clippy, dependencias, consultas SQL, pruebas y E2E. Se detiene en la primera falla; al final dice **Todo en orden**.

```sh
zsh -ic 'herramientas/revisar.sh'
```

Las capturas de pantalla quedan en `e2e/capturas/` (`ipad-mini` es donde más se rompe una pantalla).

---

## Cuando algo no arranca

| Síntoma | Qué hacer |
|---|---|
| `migration … was previously applied but has been modified` o `different checksum` | Se editó una migración ya aplicada (permitido mientras no haya producción). Recrear la base con `herramientas/dev-db.sh` y volver a crear los usuarios (pasos 5 y 6 de arriba). |
| `falta .secretos/dev.env` | Falta el paso 3 de "Una sola vez". |
| `connection refused` en el puerto 5432 | Postgres está apagado: `podman start postgres`. |
| `address already in use` en el 5100 | Otra copia sigue corriendo: detenerla, o usar otro puerto (`PORT=5101`). |
| Al entrar pide sesión otra vez | Normal después de recrear la base o de un cambio en la sesión. |

Más detalle (imagen de contenedor, variables de entorno, logs): la sección **Comandos** de [`CLAUDE.md`](CLAUDE.md).
