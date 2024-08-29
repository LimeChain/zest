use borsh::de::BorshDeserialize;
use solana_program_test::{
    processor, tokio, BanksClientError, ProgramTest, ProgramTestContext,
};
use solana_sdk::{
    account::AccountSharedData, pubkey::Pubkey, signature::Keypair,
    signer::Signer, transaction::Transaction,
};

#[tokio::test]
async fn test_program() {
    let mut validator = ProgramTest::default();
    validator.add_program("counter", counter::ID, processor!(counter::process_instruction));

    let steve = add_account(&mut validator);
    let mut context = validator.start_with_context().await;

    {
        bump_counter(&mut context, &steve)
            .await
            .unwrap();

        let steve_state_expected = context
            .banks_client
            .get_account(steve.pubkey())
            .await
            .unwrap()
            .unwrap();

        let counter = counter::state::Counter::deserialize(&mut steve_state_expected.data.as_ref()).unwrap();
        assert_eq!(counter.count, 1);
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

async fn bump_counter(
    context: &mut ProgramTestContext,
    sender: &Keypair,
) -> Result<(), BanksClientError> {
    let instruction = counter::count_instruction(sender.pubkey());

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&sender.pubkey()),
        &vec![sender],
        context.banks_client.get_latest_blockhash().await?,
    );

    context.banks_client.process_transaction(transaction).await
}
