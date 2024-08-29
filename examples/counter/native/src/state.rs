use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Counter {
    pub count: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct IncrementInstruction;
