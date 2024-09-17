use borsh::BorshDeserialize;
use solana_program_test::{
    processor, tokio, BanksClientError, ProgramTest, ProgramTestContext,
};
use solana_sdk::{
    account::AccountSharedData,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

#[tokio::test]
async fn test_program() {
    let mut validator = ProgramTest::default();
    validator.add_program(
        "counter_solana_native",
        counter_solana_native::ID,
        processor!(counter_solana_native::process_instruction),
    );

    let steve = add_account(&mut validator);
    let mut context = validator.start_with_context().await;
    let counter_pda = Pubkey::find_program_address(
        &[steve.pubkey().as_ref()],
        &counter_solana_native::ID,
    )
    .0;

    assert!(context
        .banks_client
        .get_account(counter_pda)
        .await
        .unwrap()
        .is_none());

    // Increment
    {
        increment(&mut context, &steve, counter_pda).await.unwrap();

        let steve_after = context
            .banks_client
            .get_account(counter_pda)
            .await
            .unwrap()
            .unwrap();

        let data = counter_solana_native::state::Counter::deserialize(
            &mut steve_after.data.as_ref(),
        )
        .unwrap();
        assert_eq!(data.count, 1);
    }
}

fn add_account(validator: &mut ProgramTest) -> Keypair {
    let keypair = Keypair::new();
    let account = AccountSharedData::new(
        1_000_000_000,
        0,
        &solana_sdk::system_program::id(),
    );
    validator.add_account(keypair.pubkey(), account.into());
    keypair
}

async fn increment(
    context: &mut ProgramTestContext,
    sender: &Keypair,
    counter_pda: Pubkey,
) -> Result<(), BanksClientError> {
    let instruction = Instruction::new_with_bytes(
        counter_solana_native::ID,
        // NOTE: just for the discriminator, no instruction data needed
        &[0],
        vec![
            AccountMeta::new(sender.pubkey(), true),
            AccountMeta::new(counter_pda, false),
            AccountMeta::new(solana_sdk::system_program::ID, false),
        ],
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&sender.pubkey()),
        &vec![sender],
        context.banks_client.get_latest_blockhash().await?,
    );

    context.banks_client.process_transaction(transaction).await
}
