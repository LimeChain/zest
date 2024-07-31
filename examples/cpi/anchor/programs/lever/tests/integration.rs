#![allow(unused_imports)]

use anchor_lang::AccountDeserialize;
use solana_program_test::{processor, tokio, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::AccountSharedData, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

#[allow(clippy::bool_assert_comparison)]
#[tokio::test]
async fn test_program() {
    let mut validator = ProgramTest::default();
    validator.add_program("lever", lever::ID, processor!(lever::entry));

    let owner = add_account(&mut validator);
    let mut context = validator.start_with_context().await;

    let power_pda = lever::power_pda(&owner.pubkey());

    assert!(context
        .banks_client
        .get_account(power_pda)
        .await
        .unwrap()
        .is_none());

    init_lever(&mut context, &owner, power_pda).await.unwrap();

    flip_lever_local(
        &mut context,
        &owner,
        // lever.pubkey(),
        power_pda,
        "Martin".to_string(),
    )
    .await
    .unwrap();

    {
        let lever_status_expected = context
            .banks_client
            .get_account(power_pda)
            .await
            .unwrap()
            .unwrap();

        let power_status =
            lever::PowerStatus::try_deserialize(&mut lever_status_expected.data.as_ref()).unwrap();

        assert_eq!(power_status.is_on, true);
    }
}

fn add_account(validator: &mut ProgramTest) -> Keypair {
    let keypair = Keypair::new();
    let account = AccountSharedData::new(1_000_000_000, 0, &solana_sdk::system_program::id());
    validator.add_account(keypair.pubkey(), account.into());
    keypair
}

async fn init_lever(
    context: &mut ProgramTestContext,
    sender: &Keypair,
    power: Pubkey,
) -> Result<(), BanksClientError> {
    let instruction = lever::init_instruction(sender.pubkey(), power);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&sender.pubkey()),
        &vec![sender],
        context.banks_client.get_latest_blockhash().await?,
    );

    context.banks_client.process_transaction(transaction).await
}

async fn flip_lever_local(
    context: &mut ProgramTestContext,
    sender: &Keypair,
    power: Pubkey,
    who: String,
) -> Result<(), BanksClientError> {
    let instruction = lever::switch_instruction(power, who);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&sender.pubkey()),
        &vec![sender],
        context.banks_client.get_latest_blockhash().await?,
    );

    context.banks_client.process_transaction(transaction).await
}
