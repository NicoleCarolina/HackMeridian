#!/usr/bin/env bash
# Keeper: intenta ejecutar los planes DCA que ya tocan. Cualquiera puede ejecutarlos.
# Uso: DCA_ID=C... SOURCE=me MAX_ID=20 ./scripts/keeper.sh   (cron o bucle)
set -u
: "${DCA_ID:?define DCA_ID con la dirección del contrato}"
SOURCE="${SOURCE:-me}"
MAX_ID="${MAX_ID:-50}"
for id in $(seq 1 "$MAX_ID"); do
  stellar contract invoke --id "$DCA_ID" --source "$SOURCE" --network testnet \
    -- execute --plan_id "$id" >/dev/null 2>&1 \
    && echo "plan $id ejecutado" || true   # TooEarly / PlanNotActive / inexistente: se ignora
done
