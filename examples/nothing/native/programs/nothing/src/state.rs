use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct NothingStatus {
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct NothingDoNothing {
}
