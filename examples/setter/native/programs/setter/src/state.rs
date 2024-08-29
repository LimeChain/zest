use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SetterStatus {
    pub text: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SetSetter {
    pub text: String,
}
