# Barcelona Builder HQ
### Para seguir avanzando (opcional)

Pasos extra por si quieres adelantar antes del taller. No es obligatorio, pero cuanto más resuelto llegues, más tiempo vas a tener ese día para construir en vez de configurar.

---

## Wallet de Stellar (Freighter)

Te permite jugar desde el navegador con el botón "Connect Wallet". Es distinta de la cuenta `me`, que usa tu terminal para desplegar.

1. Instala [Freighter](https://www.freighter.app/) en Chrome, Brave o Firefox.
2. Crea una wallet y guarda tu frase de recuperación.
3. Cambia la red de Mainnet a **Testnet**.
4. Pulsa **Fund with Friendbot** para recibir XLM de prueba.

**Cómo comprobarlo:** deberías ver saldo mayor a 0 XLM en Freighter. Luego, en `http://localhost:5173`, prueba "Connect Wallet" → Freighter.

---

## Raven (conector de IA para Stellar)

Le da a tu asistente de IA información actualizada de Stellar. Sirve con cualquier agente compatible con MCP.

**Si usas Claude Code:** crea `.mcp.json` en la raíz de tu proyecto (`taller-stellar-bcn/.mcp.json`):

```json
{
  "mcpServers": {
    "stellar-raven": {
      "command": "npx",
      "args": ["-y", "mcp-remote@latest", "https://raven.stellar.buzz/mcp", "--transport", "http-only"]
    }
  }
}
```

Después, dentro del proyecto, ejecuta `/mcp` e inicia sesión en el navegador.

**Si usas Claude.ai:** Configuración → Connectors → añade `https://raven.stellar.buzz/mcp`.

**Si usas otro agente MCP:** añade la misma URL con el método de conexión de tu herramienta.

**Cómo comprobarlo:** pregúntale algo sobre Stellar y confirma que responde sin error de conexión.

**Extra: Stellar Skills.** Además de Raven, existe [skills.stellar.org](https://skills.stellar.org), contexto adicional para que la IA escriba mejor código de Stellar. La forma más simple, sin instalar nada, es decirle a tu agente:

> "Read skills.stellar.org before you start building on Stellar."

Si usas Claude Code y quieres instalarlo como plugin:
```bash
/plugin marketplace add stellar/stellar-dev-skill
/plugin install stellar-dev@stellar-dev
```

Si usas Cursor, agrégalo como regla con: `stellar/stellar-dev-skill`

---

## Qué hace `npm start` en este proyecto

Cuando lo corres, primero se ejecuta `stellar scaffold build --build-clients` y espera a que termine; recién después arrancan `stellar scaffold watch --build-clients` y `vite` en paralelo. Esto evita que Vite busque el cliente antes de que exista. Esto es lo que vas a ver en tu terminal:

- `Creating keys for "me"`: se genera tu cuenta `me`.
- `Account "me" funded`: se fondea con XLM de testnet.
- `cargo rustc ... Build Complete`: se compila cada contrato de Rust a WebAssembly.
- `Using network at https://soroban-testnet.stellar.org`: se despliega en testnet.
- `Successfully generated client for: guess_the_number`: se genera el cliente TypeScript que tu frontend usa para llamar al contrato.
- `Watching for changes`: terminó; si guardas cambios en un contrato, se recompila y redespliega solo.
- `[vite] ➜ Local: http://localhost:5173/`: tu frontend ya está disponible.

---

## Jugar y explorar el contrato desde la terminal

No necesitas wallet para esto. **Todos estos comandos se corren desde dentro de la carpeta de tu proyecto** (`cd taller-stellar-bcn`), no desde tu carpeta de usuario ni desde otro lado. Si no, es posible que no encuentre tu cuenta `me` ni el alias del contrato.

```bash
cd taller-stellar-bcn
```

Obtén el Contract ID de tu instancia:

```bash
stellar contract alias show guess_the_number --network testnet
```

Si el subcomando no existe en tu versión, corre `stellar contract alias --help`.

```bash
CONTRATO=<tu-contract-id>
```

Así lo guardas en tu terminal y usas `$CONTRATO` en los comandos siguientes, en vez de pegar el ID entero cada vez. Se pierde si cierras la terminal.

Si no encuentra tu cuenta `me`, revisa con `stellar keys ls` (sigue estando dentro de la carpeta del proyecto). El aviso "Simulation identified as read-only" es normal en comandos de solo lectura.

**Ver quién es el admin**

```bash
stellar contract invoke --id $CONTRATO --source-account me --network testnet -- admin
```

**Ver el saldo del contrato** (el "pozo" acumulado, en XLM de testnet)

```bash
stellar token balance --id native --account $CONTRATO --network testnet
```

**Ver tu propio saldo**

```bash
stellar token balance --id native --account $(stellar keys address me) --network testnet
```

**Adivinar un número** (`true` si aciertas, `false` si no; si fallas te cuesta 1 XLM, si aciertas te llevas todo el saldo del contrato)

```bash
stellar contract invoke --id $CONTRATO --source-account me --network testnet -- guess --a-number 5 --guesser $(stellar keys address me)
```

**Reiniciar el número secreto** (solo si eres el admin)

```bash
stellar contract invoke --id $CONTRATO --source-account me --network testnet -- reset
```
