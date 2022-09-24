use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_pack::{IsInitialized, Sealed},
    pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct StudentInfo {
    pub discriminator: String,
    pub is_initialized: bool,
    pub name: String,
    pub msg: String,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ReplyInfo {
    pub discriminator: String,
    pub is_initialized: bool,
    pub student_info: Pubkey,
    pub reply: String,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ReplyCounter {
    pub discriminator: String,
    pub is_initialized: bool,
    pub count: u64
}

impl Sealed for StudentInfo {}

impl Sealed for ReplyCounter {}

impl IsInitialized for StudentInfo {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl IsInitialized for ReplyInfo {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl IsInitialized for ReplyCounter {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl StudentInfo {
    pub const DISCRIMINATOR: &'static str = "student";

    // pub discriminator: String,
    // pub is_initialized: bool,
    // pub name: String,
    // pub msg: String,
    pub fn get_account_size(name: String, msg: String) -> usize {
        return 
            (4 + Self::DISCRIMINATOR.len())
            + 1
            + (4 + name.len())
            + (4 + msg.len())
    }
}

impl ReplyInfo {
    pub const DISCRIMINATOR: &'static str = "reply";

    // pub discriminator: String,
    // pub is_initialized: bool,
    // pub student_info: Pubkey,
    // pub reply: String,
    pub fn get_account_size(reply: String) -> usize {
        return 
            (4 + Self::DISCRIMINATOR.len())
            + 1
            + 32
            + (4 + reply.len())
    }
}

impl ReplyCounter {
    pub const DISCRIMINATOR: &'static str = "counter";

    // could be a const, but using fn for consistency
    // pub discriminator: String,
    // pub is_initialized: bool,
    // pub count: u64
    pub fn get_account_size() -> usize {
        return 
            (4 + Self::DISCRIMINATOR.len())
            + 1
            + 8
    }
}


// student -> message 1:1
// message -> reply 1:many
// replier -> message 1:many

// need to be able to page for replies, could be a lot of them
// Also possibly want to show most recent to first
// Can keep a reply counter to know how many there are, then look up reply N 
// based on a field on its account