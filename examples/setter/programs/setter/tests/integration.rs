use anchor_lang::AccountDeserialize;
use solana_program_test::{processor, tokio, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::AccountSharedData, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

#[tokio::test]
async fn test_program() {
    let mut validator = ProgramTest::default();
    validator.add_program("setter", setter::ID, processor!(setter::entry));

    let steve = add_account(&mut validator);
    let ivan = add_account(&mut validator);
    let mut context = validator.start_with_context().await;

    let steve_text = "Hello Steve";
    let ivan_text = "Hello Ivan";

    let text_pda_steve = setter::text_pda(&steve.pubkey());
    let text_pda_ivan = setter::text_pda(&ivan.pubkey());

    // Check no mailboxes exist yet
    assert!(context
        .banks_client
        .get_account(text_pda_steve)
        .await
        .unwrap()
        .is_none());

    assert!(context
        .banks_client
        .get_account(text_pda_ivan)
        .await
        .unwrap()
        .is_none());

    // Set first message
    set_text(&mut context, &steve, text_pda_steve, steve_text.to_string())
        .await
        .unwrap();

    {
        let steve_text_expected = context
            .banks_client
            .get_account(text_pda_steve)
            .await
            .unwrap()
            .unwrap();

        let text_data =
            setter::SetterState::try_deserialize(&mut steve_text_expected.data.as_ref()).unwrap();
        assert_eq!(text_data.text, steve_text);
    }

    // Set second message
    set_text(&mut context, &ivan, text_pda_ivan, ivan_text.to_string())
        .await
        .unwrap();

    {
        let ivan_text_expected = context
            .banks_client
            .get_account(text_pda_ivan)
            .await
            .unwrap()
            .unwrap();

        let text_data =
            setter::SetterState::try_deserialize(&mut ivan_text_expected.data.as_ref()).unwrap();
        assert_eq!(text_data.text, ivan_text);
    }
}

fn add_account(validator: &mut ProgramTest) -> Keypair {
    let keypair = Keypair::new();
    let account = AccountSharedData::new(1_000_000_000, 0, &solana_sdk::system_program::id());
    validator.add_account(keypair.pubkey(), account.into());
    keypair
}

async fn set_text(
    context: &mut ProgramTestContext,
    sender: &Keypair,
    text_pda: Pubkey,
    text: String,
) -> Result<(), BanksClientError> {
    let instruction = setter::set_instruction(sender.pubkey(), text_pda, text);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&sender.pubkey()),
        &vec![sender],
        context.banks_client.get_latest_blockhash().await?,
    );

    context.banks_client.process_transaction(transaction).await
}
