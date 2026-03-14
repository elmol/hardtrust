use alloy::{
    network::EthereumWallet,
    primitives::{keccak256, Address, FixedBytes},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
};
use clap::{Parser, Subcommand};

sol!(
    #[sol(rpc)]
    HardTrustRegistry,
    "../contracts/out/HardTrustRegistry.sol/HardTrustRegistry.json"
);

#[derive(Parser)]
#[command(name = "attester")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Register a device on-chain
    Register {
        /// Device serial string
        #[arg(long)]
        serial: String,
        /// Device Ethereum address
        #[arg(long)]
        device_address: Address,
        /// Deployed HardTrustRegistry contract address
        #[arg(long)]
        contract: Address,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Register {
            serial,
            device_address,
            contract,
        } => {
            let serial_hash = keccak256(serial.as_bytes());

            // Anvil account #1 private key
            let signer: PrivateKeySigner =
                "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
                    .parse()
                    .expect("valid private key");
            let wallet = EthereumWallet::from(signer);

            let provider = ProviderBuilder::new()
                .wallet(wallet)
                .connect_http("http://127.0.0.1:8545".parse().expect("valid URL"));

            let registry = HardTrustRegistry::new(contract, &provider);
            let serial_hash_bytes: FixedBytes<32> = serial_hash.into();
            let tx = registry
                .registerDevice(serial_hash_bytes, device_address)
                .send()
                .await
                .expect("failed to send transaction")
                .watch()
                .await
                .expect("failed to confirm transaction");

            println!("tx: {tx}");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore] // Requires Anvil + deployed contract
    fn register_device_on_anvil() {
        // Integration test — run manually with:
        // cargo test -p attester -- --ignored
    }
}
