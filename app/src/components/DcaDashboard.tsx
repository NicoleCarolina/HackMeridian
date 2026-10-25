import { dcaPlatform } from "@stellar-scaffold/app-lib/clients"
import { useCallback, useEffect, useState } from "react"
import { useWallet } from "../hooks/useWallet"
import "./DcaDashboard.css"

// Direcciones de los tokens (SAC) definidas en app/.env
const USDC = import.meta.env.PUBLIC_USDC_CONTRACT as string | undefined
const XLM = import.meta.env.PUBLIC_XLM_CONTRACT as string | undefined

const DECIMALS = 7n
const SCALE = 10n ** DECIMALS

/** "12.5" -> 125000000n (sin pasar por floats). */
const toUnits = (value: string): bigint => {
	const [whole = "0", frac = ""] = value.trim().split(".")
	const padded = (frac + "0".repeat(Number(DECIMALS))).slice(0, Number(DECIMALS))
	return BigInt(whole || "0") * SCALE + BigInt(padded || "0")
}

const fromUnits = (value: bigint): string => {
	const whole = value / SCALE
	const frac = (value % SCALE).toString().padStart(Number(DECIMALS), "0")
	return `${whole}.${frac.replace(/0+$/, "") || "0"}`
}

type Direction = "usdc-xlm" | "xlm-usdc"

interface PlanView {
	id: bigint
	token_in: string
	token_out: string
	amount_per_buy: bigint
	interval: bigint
	remaining: number
	next_exec: bigint
	slippage_bps: number
}

export const DcaDashboard = () => {
	const { address, signTransaction } = useWallet()
	const [plans, setPlans] = useState<PlanView[]>([])
	const [admin, setAdmin] = useState<string>()
	const [feeBps, setFeeBps] = useState<number>(0)
	const [paused, setPaused] = useState(false)
	const [fees, setFees] = useState<{ usdc: bigint; xlm: bigint }>({
		usdc: 0n,
		xlm: 0n,
	})
	const [busy, setBusy] = useState(false)
	const [message, setMessage] = useState<string>()

	const configured = Boolean(USDC && XLM)
	const isAdmin = Boolean(address && admin && address === admin)

	const refresh = useCallback(async () => {
		try {
			const cfg = (await dcaPlatform.get_config()).result
			setAdmin(cfg.admin)
			setFeeBps(cfg.fee_bps)
			setPaused(cfg.paused)
			if (USDC && XLM) {
				const [u, x] = await Promise.all([
					dcaPlatform.total_fees({ token: USDC }),
					dcaPlatform.total_fees({ token: XLM }),
				])
				setFees({ usdc: u.result, xlm: x.result })
			}
			if (address) {
				const ids = (await dcaPlatform.get_user_plans({ owner: address }))
					.result
				const loaded = await Promise.all(
					ids.map(async (id) => {
						const r = (await dcaPlatform.get_plan({ plan_id: id })).result
						return r.isOk() ? { id, ...r.unwrap() } : undefined
					}),
				)
				setPlans(loaded.filter((p): p is PlanView => p !== undefined))
			} else {
				setPlans([])
			}
		} catch (e) {
			console.error(e)
		}
	}, [address])

	useEffect(() => {
		void refresh()
	}, [refresh])

	/** Ejecuta una acción firmada y refresca el estado. */
	const run = async (label: string, send: () => Promise<unknown>) => {
		setBusy(true)
		setMessage(undefined)
		try {
			await send()
			setMessage(`${label}: listo`)
			await refresh()
		} catch (e) {
			console.error(e)
			setMessage(`${label}: error (revisa la consola)`)
		} finally {
			setBusy(false)
		}
	}

	const createPlan = async (formData: FormData) => {
		if (!address || !USDC || !XLM) return
		const direction = formData.get("direction") as Direction
		const [token_in, token_out] =
			direction === "usdc-xlm" ? [USDC, XLM] : [XLM, USDC]
		await run("Crear plan", async () => {
			const tx = await dcaPlatform.create_plan(
				{
					owner: address,
					token_in,
					token_out,
					amount_per_buy: toUnits(String(formData.get("amount"))),
					interval: BigInt(String(formData.get("interval"))),
					num_buys: Number(formData.get("buys")),
					slippage_bps: Number(formData.get("slippage")),
				},
				{ publicKey: address },
			)
			return tx.signAndSend({ signTransaction })
		})
	}

	const execute = (id: bigint) =>
		run(`Ejecutar #${id}`, async () => {
			const tx = await dcaPlatform.execute(
				{ plan_id: id },
				{ publicKey: address },
			)
			return tx.signAndSend({ signTransaction })
		})

	const cancel = (id: bigint) =>
		run(`Cancelar #${id}`, async () => {
			const tx = await dcaPlatform.cancel(
				{ plan_id: id },
				{ publicKey: address },
			)
			return tx.signAndSend({ signTransaction })
		})

	const setFee = async (formData: FormData) => {
		await run("Cambiar comisión", async () => {
			const tx = await dcaPlatform.set_fee_bps(
				{ fee_bps: Number(formData.get("fee")) },
				{ publicKey: address },
			)
			return tx.signAndSend({ signTransaction })
		})
	}

	const togglePause = () =>
		run(paused ? "Reanudar" : "Pausar", async () => {
			const tx = await dcaPlatform.set_paused(
				{ paused: !paused },
				{ publicKey: address },
			)
			return tx.signAndSend({ signTransaction })
		})

	const tokenName = (t: string) => (t === USDC ? "USDC" : t === XLM ? "XLM" : "?")

	return (
		<div className="dca">
			{!configured && (
				<div className="card dca-warning">
					Define <code>PUBLIC_USDC_CONTRACT</code> y{" "}
					<code>PUBLIC_XLM_CONTRACT</code> en <code>app/.env</code>.
				</div>
			)}
			{paused && (
				<div className="card dca-warning">
					Plataforma en pausa: solo se permite cancelar planes.
				</div>
			)}

			<div className="card">
				<h2>Crear plan DCA</h2>
				<form action={createPlan} className="dca-form">
					<label>
						Dirección
						<select name="direction" defaultValue="usdc-xlm">
							<option value="usdc-xlm">Comprar XLM con USDC</option>
							<option value="xlm-usdc">Comprar USDC con XLM</option>
						</select>
					</label>
					<label>
						Monto por compra
						<input name="amount" type="number" min="0" step="any" required />
					</label>
					<label>
						Cada (segundos)
						<input name="interval" type="number" min="60" defaultValue="86400" required />
					</label>
					<label>
						Nº de compras
						<input name="buys" type="number" min="1" max="365" defaultValue="4" required />
					</label>
					<label>
						Slippage máx. (bps)
						<input name="slippage" type="number" min="0" max="1000" defaultValue="100" required />
					</label>
					<button type="submit" disabled={busy || !address || !configured}>
						Crear plan
					</button>
				</form>
				<p className="dca-hint">
					Comisión de la plataforma: {feeBps / 100} % por ejecución.
					{!address && " Conecta tu wallet para crear un plan."}
				</p>
			</div>

			<div className="card">
				<h2>Mis planes</h2>
				{plans.length === 0 ? (
					<p>No tienes planes todavía.</p>
				) : (
					<table className="dca-table">
						<thead>
							<tr>
								<th>#</th>
								<th>Par</th>
								<th>Monto</th>
								<th>Restan</th>
								<th>Próxima</th>
								<th />
							</tr>
						</thead>
						<tbody>
							{plans.map((p) => (
								<tr key={String(p.id)}>
									<td>{String(p.id)}</td>
									<td>
										{tokenName(p.token_in)} → {tokenName(p.token_out)}
									</td>
									<td>{fromUnits(p.amount_per_buy)}</td>
									<td>{p.remaining}</td>
									<td>
										{p.remaining > 0
											? new Date(Number(p.next_exec) * 1000).toLocaleString()
											: "—"}
									</td>
									<td className="dca-actions">
										<button
											disabled={busy || p.remaining === 0 || paused}
											onClick={() => void execute(p.id)}
										>
											Ejecutar
										</button>
										<button
											disabled={busy || p.remaining === 0}
											onClick={() => void cancel(p.id)}
										>
											Cancelar
										</button>
									</td>
								</tr>
							))}
						</tbody>
					</table>
				)}
			</div>

			{isAdmin && (
				<div className="card">
					<h2>Admin</h2>
					<p>
						Comisiones acumuladas: <strong>{fromUnits(fees.usdc)} USDC</strong>{" "}
						· <strong>{fromUnits(fees.xlm)} XLM</strong>
					</p>
					<form action={setFee} className="dca-form">
						<label>
							Comisión (bps, máx. 200)
							<input name="fee" type="number" min="0" max="200" defaultValue={feeBps} />
						</label>
						<button type="submit" disabled={busy}>
							Actualizar comisión
						</button>
					</form>
					<button onClick={() => void togglePause()} disabled={busy}>
						{paused ? "Reanudar plataforma" : "Pausar plataforma"}
					</button>
				</div>
			)}

			{message && <p className="dca-hint">{message}</p>}
		</div>
	)
}
