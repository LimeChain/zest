use std::{fs, io::Write, path::Path};

use eyre::bail;

#[rustfmt::skip]
pub const TEMPLATE: &str = /* rust */ r#"
use solana_program_test::{processor, tokio, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::AccountSharedData, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

#[tokio::test]
async fn test_program() {
    let mut validator = ProgramTest::default();
    // validator.add_program("program", program::ID, processor!(program::entry));

    let account = add_account(&mut validator);
    let mut context = validator.start_with_context().await;

    // ...
}
"#;

pub fn realise(dest: impl AsRef<Path>) -> eyre::Result<()> {
    let dest_path = dest.as_ref();

    // Ensure the parent directory exists
    if let Some(parent) = dest_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // Check if the file already exists
    if dest_path.exists() {
        bail!(
            "The file already exists at the specified path: {:?}",
            dest_path
        );
    }

    // Create and write the template to the file
    let mut file = fs::File::create(dest_path)?;
    file.write_all(TEMPLATE.as_bytes())?;

    Ok(())
}
