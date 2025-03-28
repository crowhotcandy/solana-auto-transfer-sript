use anchor_lang::{
    prelude::*,
    solana_program::program_pack::Pack, __private::CLOSED_ACCOUNT_DISCRIMINATOR,
    __private::ErrorCode::AccountDidNotSerialize
};
use std::io::Write;
use spl_token::instruction::AuthorityType;
use {
    crate::*,
    anchor_lang::{
        prelude::{AccountInfo, ProgramResult},
        solana_program::{
            program::{invoke, invoke_signed},
            pubkey::Pubkey,
            rent::Rent,
            system_instruction,
        },
    },
};

///TokenTransferParams
pub struct TokenTransferParams<'a: 'b, 'b> {
    /// CHECK: source
    pub source: AccountInfo<'a>,
    /// CHECK: destination
    pub destination: AccountInfo<'a>,
    /// amount
    pub amount: u64,
    /// authority
    pub authority: AccountInfo<'a>,
    /// authority_signer_seeds
    pub authority_signer_seeds: &'b [&'b [u8]],
    /// token_program
    pub token_program: AccountInfo<'a>,
}

///TokenMintParams
pub struct TokenMintParams<'a: 'b, 'b> {
    /// mint
    pub mint: AccountInfo<'a>,
    /// to
    pub to: AccountInfo<'a>,
    /// amount
    pub amount: u64,
    /// owner
    pub owner: AccountInfo<'a>,
    /// owner_signer_seeds
    pub owner_signer_seeds: &'b [&'b [u8]],
    /// token_program
    pub token_program: AccountInfo<'a>,
}

///InitializeTokenAccount
pub struct InitializeTokenAccountParams<'a: 'b, 'b> {
    /// account
    pub account: AccountInfo<'a>,
    /// account_signer_seeds
    pub account_signer_seeds: &'b [&'b [u8]],
    /// mint
    pub mint: AccountInfo<'a>,
    /// owner
    pub owner: AccountInfo<'a>,
    /// payer
    pub payer: AccountInfo<'a>,
    /// system_program
    pub system_program: AccountInfo<'a>,
    /// token_program
    pub token_program: AccountInfo<'a>,
    /// rent
    pub rent: AccountInfo<'a>,
}

///SetAuthority
pub struct SetAuthorityParams<'a: 'b, 'b> {
    /// account
    pub account: AccountInfo<'a>,
    /// new authority
    pub new_authority: AccountInfo<'a>,
    /// authority type
    pub authority_type: AuthorityType,
    /// owner
    pub owner: AccountInfo<'a>,
    /// owner_signer_seeds
    pub owner_signer_seeds: &'b [&'b [u8]],
    /// token_program
    pub token_program: AccountInfo<'a>,
}

///CloseAccountParams
pub struct CloseAccountParams<'a: 'b, 'b> {
    /// account
    pub account: AccountInfo<'a>,
    /// destination
    pub destination: AccountInfo<'a>,
    /// owner
    pub owner: AccountInfo<'a>,
    /// owner_signer_seeds
    pub owner_signer_seeds: &'b [&'b [u8]],
    /// token_program
    pub token_program: AccountInfo<'a>,
}

pub fn spl_token_transfer(params: TokenTransferParams<'_, '_>) -> ProgramResult {
    let TokenTransferParams {
        source,
        destination,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;

    let result = invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, destination, authority, token_program],
        &[authority_signer_seeds],
    );

    result.map_err(|_| ErrorCode::TokenTransferFailed.into())
}

pub fn spl_token_mint(params: TokenMintParams<'_, '_>) -> ProgramResult {
    let TokenMintParams {
        mint,
        to,
        amount,
        owner,
        owner_signer_seeds,
        token_program,
    } = params;

    let result = invoke_signed(
        &spl_token::instruction::mint_to(
            token_program.key,
            mint.key,
            to.key,
            owner.key,
            &[],
            amount,
        )?,
        &[mint, to, owner, token_program],
        &[owner_signer_seeds],
    );

    result.map_err(|_| ErrorCode::TokenMintFailed.into())
}

pub fn spl_init_token_account(params: InitializeTokenAccountParams<'_, '_>) -> ProgramResult {
    let InitializeTokenAccountParams {
        account,
        account_signer_seeds,
        mint,
        owner,
        payer,
        system_program,
        token_program,
        rent,
    } = params;

    create_pda_account(
        &payer,
        spl_token::state::Account::LEN,
        token_program.key,
        &system_program,
        &account,
        account_signer_seeds,
    )?;

    let result = invoke(
        &spl_token::instruction::initialize_account(
            token_program.key,
            account.key,
            mint.key,
            owner.key,
        )?,
        &[account, mint, owner, token_program, rent],
    );

    result.map_err(|_| ErrorCode::InitializeTokenAccountFailed.into())
}

pub fn spl_set_authority(params: SetAuthorityParams<'_, '_>) -> ProgramResult {
    let SetAuthorityParams {
        account,
        new_authority,
        authority_type,
        owner,
        owner_signer_seeds,
        token_program,
    } = params;

    let result = invoke_signed(
        &spl_token::instruction::set_authority(
            token_program.key,
            account.key,
            Some(new_authority.key),
            authority_type,
            owner.key,
            &[],
        )?,
        &[account, new_authority, owner, token_program],
        &[owner_signer_seeds],
    );

    result.map_err(|_| ErrorCode::SetAccountAuthorityFailed.into())
}

pub fn spl_close_account(params: CloseAccountParams<'_, '_>) -> ProgramResult {
    let CloseAccountParams {
        account,
        destination,
        owner,
        owner_signer_seeds,
        token_program,
    } = params;

    let result = invoke_signed(
        &spl_token::instruction::close_account(
            token_program.key,
            account.key,
            destination.key,
            owner.key,
            &[],
        )?,
        &[account, destination, owner, token_program],
        &[owner_signer_seeds],
    );

    result.map_err(|_| ErrorCode::CloseAccountFailed.into())
}

pub fn create_pda_account<'a>(
    payer: &AccountInfo<'a>,
    space: usize,
    owner: &Pubkey,
    system_program: &AccountInfo<'a>,
    new_pda_account: &AccountInfo<'a>,
    new_pda_signer_seeds: &[&[u8]],
) -> ProgramResult {
    let rent = Rent::get()?;

    if new_pda_account.lamports() > 0 {
        let required_lamports = rent
            .minimum_balance(space)
            .max(1)
            .saturating_sub(new_pda_account.lamports());

        if required_lamports > 0 {
            invoke(
                &system_instruction::transfer(payer.key, new_pda_account.key, required_lamports),
                &[
                    payer.clone(),
                    new_pda_account.clone(),
                    system_program.clone(),
                ],
            )?;
        }

        invoke_signed(
            &system_instruction::allocate(new_pda_account.key, space as u64),
            &[new_pda_account.clone(), system_program.clone()],
            &[new_pda_signer_seeds],
        )?;

        invoke_signed(
            &system_instruction::assign(new_pda_account.key, owner),
            &[new_pda_account.clone(), system_program.clone()],
            &[new_pda_signer_seeds],
        )
    } else {
        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                new_pda_account.key,
                rent.minimum_balance(space).max(1),
                space as u64,
                owner,
            ),
            &[
                payer.clone(),
                new_pda_account.clone(),
                system_program.clone(),
            ],
            &[new_pda_signer_seeds],
        )
    }
}

pub fn close<'info>(info: AccountInfo<'info>, sol_destination: AccountInfo<'info>) -> ProgramResult {
    // Transfer tokens from the account to the sol_destination.
    let dest_starting_lamports = sol_destination.lamports();
    **sol_destination.lamports.borrow_mut() =
        dest_starting_lamports.checked_add(info.lamports()).unwrap();
    **info.lamports.borrow_mut() = 0;

    // Mark the account discriminator as closed.
    let mut data = info.try_borrow_mut_data()?;
    let dst: &mut [u8] = &mut data;
    let mut cursor = std::io::Cursor::new(dst);
    cursor
        .write_all(&CLOSED_ACCOUNT_DISCRIMINATOR)
        .map_err(|_| AccountDidNotSerialize)?;
    Ok(())
}
