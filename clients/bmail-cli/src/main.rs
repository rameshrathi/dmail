use anyhow::Result;
use bmail_chain::{BlockchainClient, LocalChainSimulator};
use bmail_crypto::WalletKeyPair;
use bmail_sdk::BmailClient;
use bmail_storage::MemoryStorageAdapter;
use clap::{Parser, Subcommand};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "bmail")]
#[command(about = "Blockchain End-to-End Encrypted Mail CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run an end-to-end multi-party E2EE mail demonstration.
    Demo,
    /// Generate a fresh Solana-compatible wallet keypair.
    GenerateWallet,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateWallet => {
            let wallet = WalletKeyPair::generate();
            println!("New Solana-Compatible Wallet Generated:");
            println!("  Address (Base58): {}", wallet.address());
            println!("  Public Key (Hex): {:02x?}", wallet.public_key_bytes());
        }
        Commands::Demo => {
            println!("============================================================");
            println!("           B-MAIL: BLOCKCHAIN E2EE MAIL DEMO                ");
            println!("============================================================\n");

            // 1. Initialize shared local blockchain simulator & storage
            let chain = Arc::new(LocalChainSimulator::new());
            let storage = Arc::new(MemoryStorageAdapter::new());

            // 2. Set up Alice & Bob clients
            let alice_wallet = WalletKeyPair::generate();
            let bob_wallet = WalletKeyPair::generate();
            let charlie_wallet = WalletKeyPair::generate();
            let charlie_seed = charlie_wallet.to_bytes();

            let alice = BmailClient::new(alice_wallet, chain.clone(), storage.clone())?;
            let bob = BmailClient::new(bob_wallet, chain.clone(), storage.clone())?;
            let charlie = BmailClient::new(charlie_wallet, chain.clone(), storage.clone())?;

            println!("[Step 1] Initializing Users and On-Chain Registration");
            println!("  Alice Address: {}", alice.wallet_address());
            println!("  Bob Address:   {}", bob.wallet_address());
            println!("  Charlie:       {}\n", charlie.wallet_address());

            // Bob registers his public encryption key on-chain
            bob.register_on_chain()?;
            println!("✓ Bob registered public encryption key on-chain.\n");

            // 3. Alice composes and sends encrypted mail to Bob
            println!("[Step 2] Alice Composing and Encrypting Mail to Bob");
            let subject = "Solana Validator Staking Private Agreement";
            let body =
                "Hi Bob, here are the confidential parameters for node 7xK3: Stake=10,000 SOL.";
            println!("  Subject: {}", subject);
            println!("  Body:    {}\n", body);

            let msg_id = alice.send_mail(&bob.wallet_address(), subject, body, vec![])?;
            println!("✓ Mail dispatched to blockchain!");
            println!("  Message ID: {:02x?}\n", &msg_id[..8]);

            // 4. Bob retrieves and decrypts mail from blockchain
            println!("[Step 3] Bob Fetching and Decrypting Mail");
            let inbox = bob.fetch_inbox()?;
            assert_eq!(inbox.len(), 1);
            let decrypted = &inbox[0];

            println!("✓ Decryption Succeeded on Bob's Device!");
            println!("  From:    {}", decrypted.payload.sender);
            println!("  Subject: {}", decrypted.payload.subject);
            println!("  Body:    {}\n", decrypted.payload.body);

            // 5. Security Proof: Charlie (Observer) attempts to read Bob's mail
            println!("[Step 4] Security Test: Charlie Attempts Unauthorized Decryption");
            let charlie_inbox = charlie.fetch_inbox()?;
            println!("  Charlie's inbox count: {}", charlie_inbox.len());

            // Even if Charlie intercepts Bob's encrypted dispatch record from the chain
            let intercepted_record = chain.fetch_inbox(&bob.wallet_address())?[0].clone();
            println!("  Charlie intercepted Bob's on-chain dispatch record:");
            println!("    Storage URI:  {}", intercepted_record.storage_uri);
            println!(
                "    Payload Hash: {:02x?}...",
                &intercepted_record.payload_hash[..8]
            );

            // Charlie tries to unwrap the CEK using Charlie's private key
            let charlie_wallet2 = WalletKeyPair::from_seed(&charlie_seed);
            let charlie_sig = charlie_wallet2.sign(b"B-MAIL-ENCRYPTION-KEY-V1");
            let charlie_enc_keypair =
                bmail_crypto::EncryptionKeyPair::derive_from_signature(&charlie_sig)?;

            let attack = bmail_crypto::unwrap_cek(
                &charlie_enc_keypair,
                &intercepted_record.ephemeral_pubkey,
                &intercepted_record.wrapped_cek,
            );
            assert!(attack.is_err());

            println!("  Charlie attempting CEK decapsulation with unauthorized private key...");
            println!("  [Result]: Mathematical failure! Only Bob's private key can decrypt.\n");

            println!("============================================================");
            println!("           B-MAIL E2EE DEMO COMPLETED SUCCESSFULLY!         ");
            println!("============================================================");
        }
    }

    Ok(())
}
