# Barcelona Builder HQ
### Guía de preparación

Por favor completa estos pasos **antes** del día del taller. Así aprovechamos todo el tiempo del evento para construir, en vez de instalar cosas.

---

## Qué necesitas

- Un ordenador con conexión a internet
- Una cuenta activa en algún asistente de IA compatible con MCP (Claude, Cursor, u otro). El plan gratuito de Claude.ai, por ejemplo, es suficiente

No necesitas experiencia previa en blockchain ni Stellar.

---

## Paso 1: Instalar las herramientas

```bash
# Rust (el lenguaje del contrato)
curl https://sh.rustup.rs -sSf | sh

# Stellar CLI
curl -fsSL https://github.com/stellar/stellar-cli/raw/main/install.sh | sh

# Scaffold Stellar
cargo install --locked stellar-scaffold-cli
```

Instala también **Node.js** (versión LTS) desde [nodejs.org](https://nodejs.org).

**Verifica:**
```bash
rustc --version
node --version
npm --version
stellar --version
```

## Paso 2: Descargar el proyecto base

```bash
git clone https://github.com/janijesse/taller-stellar-bcn.git
cd taller-stellar-bcn
npm install
```

## Paso 3: Configurar el frontend

Tu frontend necesita su propio archivo de configuración (ya viene con los cambios necesarios en el repositorio, solo falta que crees tu copia local):

```bash
cp app/.env.example app/.env
```

## Paso 4: Verificar que todo funciona

```bash
npm start
```

La primera vez tarda un poco: crea tu cuenta de prueba, la fondea, despliega tu propio contrato en testnet y levanta la interfaz web. Cuando veas algo como:

```
[vite] ➜  Local:   http://localhost:5173/
```

abre esa dirección en tu navegador.

---

## Checklist final

- [ ] `npm start` corre sin errores y la página carga en `http://localhost:5173`
- [ ] Tengo cuenta activa en algún asistente de IA compatible con MCP
- [ ] Traigo mi ordenador 
- [ ] (Opcional) Traigo una idea de proyecto, aunque sea muy general

---

Hay una segunda guía opcional, [Para seguir avanzando](seguir-avanzando.md), con más contexto y pasos extra. No es obligatoria, pero si la completas antes, el día del taller tendrás más tiempo para construir en vez de configurar.

---

## Recursos

- [Documentación de Stellar](https://developers.stellar.org)
- [Scaffold Stellar](https://developers.stellar.org/docs/tools/scaffold-stellar)
- [Raven](https://raven.stellar.buzz): el conector de IA que usaremos en el taller

Nos vemos en el taller.
