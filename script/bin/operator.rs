use anyhow::Result;
use helios_consensus_core::consensus_spec::MainnetConsensusSpec;
use helios_ethereum::consensus::Inner;
use helios_ethereum::rpc::http_rpc::HttpRpc;
use helios_ethereum::rpc::ConsensusRpc;
use log::info;
use sp1_helios_primitives::types::ProofInputs;
use sp1_helios_script::*;
use sp1_sdk::{EnvProver, ProverClient, SP1ProofWithPublicValues, SP1ProvingKey, SP1Stdin};
use std::env;

const ELF: &[u8] = include_bytes!("../../elf/sp1-helios-elf");

struct SP1HeliosOperator {
    client: EnvProver,
    pk: SP1ProvingKey,
}

impl SP1HeliosOperator {
    pub async fn new() -> Self {
        dotenv::dotenv().ok();

        let client = ProverClient::from_env();
        let (pk, _) = client.setup(ELF);

        Self { client, pk }
    }

    /// Fetch values and generate an 'update' proof for the SP1 Helios contract.
    async fn request_update(
        &self,
        client: Inner<MainnetConsensusSpec, HttpRpc>,
    ) -> Result<Option<SP1ProofWithPublicValues>> {
        // the head we are trying to prove
        let head: u64 = 11558913;

        let mut stdin = SP1Stdin::new();

        // Setup client.
        let updates = get_updates(&client).await;
        println!("About to prove {:?} light client updates!", updates.len());
        let finality_update = client.rpc.get_finality_update().await.unwrap();

        // Check if contract is up to date
        let latest_block = finality_update.finalized_header().beacon().slot;
        if latest_block <= head {
            info!("Contract is up to date. Nothing to update.");
            return Ok(None);
        }
        // Create program inputs
        let expected_current_slot = client.expected_current_slot();
        let inputs = ProofInputs {
            updates,
            finality_update,
            expected_current_slot,
            store: client.store.clone(),
            genesis_root: client.config.chain.genesis_root,
            forks: client.config.forks.clone(),
        };
        let encoded_proof_inputs = serde_cbor::to_vec(&inputs)?;
        stdin.write_slice(&encoded_proof_inputs);

        // Generate proof.
        let proof = self.client.prove(&self.pk, &stdin).groth16().run()?;

        info!("Attempting to update to new head block: {:?}", latest_block);
        Ok(Some(proof))
    }

    /// Start the operator.
    async fn run(&mut self) {
        info!("Starting SP1 Helios operator");
        // slot multiple of 8192
        let slot: u64 = 11558912;
        let checkpoint = get_checkpoint(slot).await.unwrap();
        // Get the client from the checkpoint
        let client = get_client(checkpoint).await.unwrap();

        // Request an update
        self.request_update(client).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    dotenv::dotenv().ok();
    env_logger::init();

    let mut operator = SP1HeliosOperator::new().await;
    operator.run().await;
}
