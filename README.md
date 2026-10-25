![Stellar](https://img.shields.io/badge/Stellar-black?style=for-the-badge)

```
╔══════════════════════════════════════════╗
║                                            ║
║       🕹️   BARCELONA BUILDER HQ   🕹️      ║
║                                            ║
╚══════════════════════════════════════════╝
```

![Network](https://img.shields.io/badge/network-testnet-9945FF?style=for-the-badge)
![Built with](https://img.shields.io/badge/built%20with-Soroban-000000?style=for-the-badge)
![Scaffold](https://img.shields.io/badge/Scaffold%20Stellar-grey?style=for-the-badge)
![Status](https://img.shields.io/badge/status-ready%20to%20hack-brightgreen?style=for-the-badge)

---

👉 **Antes del taller, sigue la [guía de preparación](docs/preparacion.md).**

---

## Qué es esto?

Plataforma de **DCA (compras recurrentes) en Stellar Testnet**: el usuario deposita USDC (o XLM) y programa compras periódicas del otro activo. La plataforma cobra una **comisión por ejecución** (0,5 % por defecto, máx. 2 %) que va a una tesorería.

```
create_plan  → el usuario deposita monto × nº de compras (escrow en el contrato)
execute      → cualquiera (bot/keeper) la dispara cuando toca:
               comisión → tesorería, resto → swap vía router → el usuario recibe
cancel       → devuelve el saldo no ejecutado (siempre permitido, incluso en pausa)
admin        → set_fee_bps (≤ 200), set_treasury, set_router, set_paused
```

Contrato: `contracts/dca-platform/src/lib.rs`. El swap está detrás de la interfaz `router.rs` (compatible con Soroswap); los tests usan un router simulado.

## Cómo correrlo

```
npm install
cp app/.env.example app/.env   # completa PUBLIC_USDC_CONTRACT y PUBLIC_XLM_CONTRACT
npm start
cargo test -p dca-platform     # tests del contrato (9)
```

Keeper (ejecuta los planes que ya tocan): `DCA_ID=C... ./scripts/keeper.sh`

## Estado y pendientes (léelo antes de la demo)

- **Tests del contrato: pasan** (9 OK — crear, ejecutar, cancelar, ejecutar antes de tiempo, comisión, slippage, pausa, validaciones, tope de comisión).
- **Router de Soroswap en testnet: VERIFICADO.** Dirección fijada en `environments.toml`:
  `CCJUD55AG6W5HAI5LRVNKAE5WDP5XGZBUDS5WNTIVDU7O264UZZE7BRD` (fuente: [soroswap/core testnet.contracts.json](https://github.com/soroswap/core/blob/main/public/testnet.contracts.json)). Firmas del trait `router.rs` verificadas contra el router real de Soroswap (`router_get_amounts_out`, `router_pair_for`, `swap_exact_tokens_for_tokens`).
- **Direcciones USDC/XLM: FIJADAS.** Usamos los tokens que Soroswap deploya en testnet (los que tienen pools de liquidez), **no** el USDC real de Circle:
  - XLM (nativo wrapped): `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`
  - USDC (Soroswap mint): `CB3TLW74NBIOT3BUWOZ3TUM6RFDF6A4GVIRUQRQZABG5KPOUL4JJOV2F`
  - Catálogo completo: [soroswap/core tokens.json](https://github.com/soroswap/core/blob/main/public/tokens.json). El USDC real de Circle en testnet (emisor `GBBD47IF...`) no tiene pool en Soroswap, por eso usamos el de Soroswap.
- **Verificar pool USDC/XLM en Soroswap testnet antes de la demo:** `stellar contract invoke --id CCJUD55AG6W5HAI5LRVNKAE5WDP5XGZBUDS5WNTIVDU7O264UZZE7BRD --network testnet -- router_pair_for --token_a CB3TLW74NBIOT3BUWOZ3TUM6RFDF6A4GVIRUQRQZABG5KPOUL4JJOV2F --token_b CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`. Si devuelve una address, el par existe. Luego revisa reservas con `stellar contract invoke --id <PAIR> --network testnet -- get_reserves`.
- **Frontend: no compilado ni probado.** Está escrito contra el cliente TypeScript que genera `npm start`; puede requerir ajustes de tipos.
- **Slippage:** la cotización se toma en la misma transacción; en mainnet conviene contrastarla con un oráculo (p. ej. Reflector).
- **Solo testnet.** Un contrato que custodia fondos necesita auditoría antes de mainnet.

---

## Stack técnico

```
contrato  →  Rust + Soroban
frontend  →  React + Vite
red       →  Stellar Testnet
base      →  Scaffold Stellar
```

---

## Estructura del proyecto

```
taller-stellar-bcn/
├── contracts/
│   └── dca-platform/       ← contrato DCA
├── app/                    ← el frontend
│   ├── .env                ← tu config local (no se sube)
│   └── .env.example
├── environments.toml        ← qué red y qué cuenta usar
└── package.json
```

---

## Problemas comunes

**La pantalla no carga la primera vez** → espera a que la terminal diga `Successfully generated client for: dca_platform`, luego refresca.

**No tienes wallet** → puedes jugar igual desde la terminal, invocando el contrato directamente.

---

## Créditos

Base construida sobre [Scaffold Stellar](https://developers.stellar.org/docs/tools/scaffold-stellar), para Barcelona Builder HQ.
