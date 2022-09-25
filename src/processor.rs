use crate::error::StudentIntroError;
use crate::instruction::IntroInstruction;
use crate::state::{ReplyCounter, ReplyInfo, StudentInfo};
use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use std::convert::TryInto;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = IntroInstruction::unpack(instruction_data)?;
    match instruction {
        IntroInstruction::InitUserInput { name, message } => {
            add_student_intro(program_id, accounts, name, message)
        }
        IntroInstruction::UpdateStudentIntro { name, message } => {
            update_student_intro(program_id, accounts, name, message)
        }
        IntroInstruction::Reply { reply } => add_reply(program_id, accounts, reply),
    }
}

pub fn add_student_intro(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    message: String,
) -> ProgramResult {
    msg!("Adding student intro...");
    msg!("Name: {}", name);
    msg!("Message: {}", message);
    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let user_account = next_account_info(account_info_iter)?;
    let pda_counter: &AccountInfo = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    let (pda, bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref()], program_id);
    if pda != *user_account.key {
        msg!("Invalid seeds for PDA");
        return Err(StudentIntroError::InvalidPDA.into());
    }

    let total_len: usize = 1 + (4 + name.len()) + (4 + message.len());
    if total_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(StudentIntroError::InvalidDataLength.into());
    }
    let account_len: usize = 1000;

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            user_account.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            user_account.clone(),
            system_program.clone(),
        ],
        &[&[initializer.key.as_ref(), &[bump_seed]]],
    )?;

    msg!("PDA created: {}", pda);

    msg!("unpacking state account");
    let mut account_data =
        try_from_slice_unchecked::<StudentInfo>(&user_account.data.borrow()).unwrap();
    msg!("borrowed account data");

    msg!("checking if account is already initialized");
    if account_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    account_data.discriminator = StudentInfo::DISCRIMINATOR.to_string();
    account_data.name = name;
    account_data.msg = message;
    account_data.is_initialized = true;
    msg!("serializing account");
    account_data.serialize(&mut &mut user_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    let counter_len = ReplyCounter::get_account_size();
    let rent = Rent::get()?;
    let counter_rent_lamports = rent.minimum_balance(counter_len);

    let (reply_counter_pda, reply_counter_bump) =
        Pubkey::find_program_address(&[pda.as_ref(), b"reply"], program_id);
    if reply_counter_pda != *pda_counter.key {
        msg!("Invalid counter PDA seeds");
        return Err(ProgramError::InvalidArgument);
    }

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            pda_counter.key,
            counter_rent_lamports,
            counter_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            pda_counter.clone(),
            system_program.clone(),
        ],
        &[&[pda.as_ref(), b"reply", &[reply_counter_bump]]],
    )?;

    let mut counter_data =
        try_from_slice_unchecked::<ReplyCounter>(&pda_counter.data.borrow()).unwrap();

    msg!("checking if counter account is already initialized");
    if counter_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    counter_data.discriminator = ReplyInfo::DISCRIMINATOR.to_string();
    counter_data.count = 0;
    counter_data.is_initialized = true;
    msg!("reply count: {}", counter_data.count);

    counter_data.serialize(&mut &mut pda_counter.data.borrow_mut()[..])?;

    Ok(())
}

pub fn update_student_intro(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    message: String,
) -> ProgramResult {
    msg!("Updating student intro...");
    msg!("Name: {}", name);
    msg!("Message: {}", message);
    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let user_account = next_account_info(account_info_iter)?;

    msg!("unpacking state account");
    let mut account_data =
        try_from_slice_unchecked::<StudentInfo>(&user_account.data.borrow()).unwrap();
    msg!("borrowed account data");

    msg!("checking if account is initialized");
    if !account_data.is_initialized() {
        msg!("Account is not initialized");
        return Err(StudentIntroError::UninitializedAccount.into());
    }

    if user_account.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    let (pda, _bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref()], program_id);
    if pda != *user_account.key {
        msg!("Invalid seeds for PDA");
        return Err(StudentIntroError::InvalidPDA.into());
    }

    let update_len: usize = 1 + (4 + account_data.name.len()) + (4 + message.len());
    if update_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(StudentIntroError::InvalidDataLength.into());
    }

    // should already have the right discriminator
    // account_data.discriminator = StudentInfo::DISCRIMINATOR.to_string();
    account_data.name = account_data.name;
    account_data.msg = message;
    msg!("serializing account");
    account_data.serialize(&mut &mut user_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    Ok(())
}

pub fn add_reply(program_id: &Pubkey, accounts: &[AccountInfo], reply: String) -> ProgramResult {
    msg!("Adding reply to student intro...");
    msg!("Reply: {}", reply);
    let account_info_iter = &mut accounts.iter();

    let replier = next_account_info(account_info_iter)?;
    let pda_student: &AccountInfo = next_account_info(account_info_iter)?;
    let pda_counter: &AccountInfo = next_account_info(account_info_iter)?;
    let pda_reply: &AccountInfo = next_account_info(account_info_iter)?;
    let system_program: &AccountInfo = next_account_info(account_info_iter)?;

    if pda_counter.owner != system_program.key {
        msg!("Invalid owner for counter PDA");
        return Err(ProgramError::IllegalOwner);
    }

    let counter_data =
        try_from_slice_unchecked::<ReplyCounter>(&pda_counter.data.borrow()).unwrap();

    let account_len: usize = ReplyInfo::get_account_size(reply.clone());

    let rent = Rent::get()?;
    let account_lamports = rent.minimum_balance(account_len);

    let (reply_pda, reply_bump_seed) = Pubkey::find_program_address(
        &[
            pda_student.key.as_ref(),
            counter_data.count.to_be_bytes().as_ref(),
        ],
        program_id,
    );
    if reply_pda != *pda_reply.key {
        msg!("Passed in PDA does not match computed PDA");
        return Err(StudentIntroError::InvalidPDA.into());
    }

    invoke_signed(
        &system_instruction::create_account(
            replier.key,
            pda_reply.key,
            account_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[replier.clone(), pda_reply.clone(), system_program.clone()],
        &[&[
            pda_student.key.as_ref(),
            counter_data.count.to_be_bytes().as_ref(),
            &[reply_bump_seed],
        ]],
    )?;

    msg!("Created reply account");

    let mut reply_data: ReplyInfo =
        try_from_slice_unchecked::<ReplyInfo>(&pda_reply.data.borrow()).unwrap();
    msg!("Checking if reply is alreayd initialized");
    if reply_data.is_initialized {
        msg!("Reply alreayd initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let mut counter_data: ReplyCounter =
        try_from_slice_unchecked::<ReplyCounter>(&pda_counter.data.borrow()).unwrap();

    reply_data.discriminator = ReplyInfo::DISCRIMINATOR.to_string();
    reply_data.reply = reply;
    reply_data.student_info = *pda_student.key;
    reply_data.is_initialized = true;
    reply_data.serialize(&mut &mut pda_reply.data.borrow_mut()[..])?;

    msg!("Reply Count: {}", counter_data.count);
    counter_data.discriminator = ReplyCounter::DISCRIMINATOR.to_string();
    counter_data.count += 1;
    counter_data.serialize(&mut &mut pda_counter.data.borrow_mut()[..])?;

    Ok(())
}
