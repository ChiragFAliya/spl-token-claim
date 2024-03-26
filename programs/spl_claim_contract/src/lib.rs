use std::vec;

use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

declare_id!("6mnNGTZiWSp4bJsCxea1bvi4DTsD1opZhDwbfneYzbsD");

#[program]
pub mod spl_claim_contract {

    const ADMIN: &str = "4rdE7Ub5w5bc9QvFoYLRVdT3B6aLQUiD84hezHW2JEwi";
    use std::str::FromStr;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        if !(ctx.accounts.owner.key == &Pubkey::from_str(ADMIN).unwrap()) {
            return Err(Errors::NotAuthorized.into());
        }
        Ok(())
    }

    pub fn update_users(
        ctx: Context<UpdateUser>,
        amounts: Vec<u64>,
        users: Vec<Pubkey>,
    ) -> Result<()> {
        if !(ctx.accounts.owner.key == &Pubkey::from_str(ADMIN).unwrap()) {
            return Err(Errors::NotAuthorized.into());
        }
        if amounts.len() != users.len() {
            return Err(Errors::InvalidInput.into());
        }
        let mut current_index = ctx.accounts.global.total_users;
        for (_, (amount, user)) in amounts.iter().zip(users.iter()).enumerate() {
            ctx.accounts.user_list.user[current_index as usize] = *user;
            ctx.accounts.user_list.token[current_index as usize] = *amount;

            current_index += 1;

            ctx.accounts.global.claimable_tokens += *amount;
        }
        // let mut u = ctx.accounts.user_list.user.get_mut(current_index as usize).unwrap();
        // ctx.accounts.user_list.user.copy_from_slice(src)

        ctx.accounts.global.total_users += users.len() as u64;

        Ok(())
    }

    // pub fn check_eligibility(ctx: Context<CheckEligibility>) -> Result<()> {
    //     let users = ctx.accounts.user_list.user;
    //     if !users.contains(ctx.accounts.user.key) {
    //         return err!(Errors::NotEligible);
    //     }

    //     Ok(())
    // }

    pub fn claim_token(ctx: Context<ClaimToken>, bump: u8, index: u64) -> Result<()> {
        let users = ctx.accounts.user_list.user;
        let owner = ctx.accounts.user.key;

        if owner != &users[index as usize] {
            return Err(Errors::InvalidIndex.into());
        }

        let amount = ctx.accounts.user_list.token[index as usize];
        if amount == 0 {
            return Err(Errors::NotEligible.into());
        }
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.list_ata.to_account_info().clone(),
                    to: ctx.accounts.user_ata.to_account_info().clone(),
                    authority: ctx.accounts.user_list.to_account_info().clone(),
                },
                &[&[b"list", &[bump]]],
            ),
            amount,
        )?;
        ctx.accounts.user_list.token[index as usize] = 0;

        ctx.accounts.global.claimable_tokens -= amount;
        ctx.accounts.global.claimed_tokens += amount;

        Ok(())
    }

}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = owner,seeds = ["list".as_ref()] ,bump,space = 8 + 2008)]
    pub user_list: Box<Account<'info, User>>,
    #[account(init_if_needed,payer=owner,associated_token::mint= mint, associated_token::authority = user_list)]
    pub list_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(init, payer = owner,seeds = ["global".as_ref()] ,bump,space = 8 + 24)]
    pub global: Account<'info, Global>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
    ///CHECK
    pub associated_token_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateUser<'info> {
    #[account(mut)]
    pub user_list: Box<Account<'info, User>>,
    #[account(mut)]
    pub global: Account<'info, Global>,
    #[account(mut)]
    pub owner: Signer<'info>,
    // pub user_pubkey: AccountInfo<'info>,
}

// #[derive(Accounts)]
// pub struct CheckEligibility<'info> {
//     #[account(mut)]
//     pub user_list: Account<'info, User>,
//     #[account(mut)]
//     pub user: Signer<'info>,
// }

#[derive(Accounts)]
pub struct ClaimToken<'info> {
    #[account(mut)]
    pub user_list: Box<Account<'info, User>>,
    #[account(mut)]
    pub global: Account<'info, Global>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(init_if_needed,payer=user,associated_token::mint= mint, associated_token::authority = user)]
    pub user_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub list_ata: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    ///CHECK
    pub associated_token_program: AccountInfo<'info>,
}

#[account]
pub struct Global {
    claimable_tokens: u64,
    claimed_tokens: u64,
    total_users: u64,
}

#[account]
pub struct User {
    user: [Pubkey; 50],
    token: [u64; 50],
}

#[error_code]
pub enum Errors {
    #[msg("you are not eligible!")]
    NotEligible,
    #[msg("you are not authorized to this !")]
    NotAuthorized,
    #[msg("Invalid input!")]
    InvalidInput,
    #[msg("Invalid index!")]
    InvalidIndex,
}
