import { labPrefix } from "@stellar-scaffold/app-lib"
import React from "react"
import { Link } from "react-router-dom"
import { DcaDashboard } from "../components/DcaDashboard"
import styles from "./Home.module.css"

const Home: React.FC = () => (
	<div className={styles.Home}>
		<div>
			<h1>DCA en Stellar</h1>
			<p>
				Plataforma de compras recurrentes entre USDC y XLM sobre Stellar
				Testnet. La plataforma cobra una comisión por cada ejecución.
			</p>
		</div>

		<DcaDashboard />

		<section className={styles.cards}>
			<div className="card">
				<p>
					Invoke your smart contract using the{" "}
					<Link to="/debug">Contract Explorer</Link>
				</p>
			</div>
			<div className="card">
				<p>
					Browse your local transactions with the{" "}
					<Link to={labPrefix()}>Transaction Explorer</Link>
				</p>
			</div>
		</section>
	</div>
)

export default Home
