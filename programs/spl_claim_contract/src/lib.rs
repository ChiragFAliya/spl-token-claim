use std::vec;

use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

declare_id!("EpomsAFj8dXtKBgX5jqAQCYeQNWAx9jQ6hEw2ASck2YN");

#[program]
pub mod spl_claim_contract {


    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn update_users(ctx: Context<UpdateUser>, index: u8, amount: u32) -> Result<()> {
        ctx.accounts.user_list.user[index as usize] = ctx.accounts.user_pubkey.key();
        ctx.accounts.user_list.token[index as usize] = amount;
    
        Ok(())
    }
    

    pub fn check_eligibility(ctx: Context<CheckEligibility>) -> Result<()> {
        let users = ctx.accounts.user_list.user;
        if !users.contains(ctx.accounts.user.key) {
            return err!(Errors::NotEligible);
        }

        Ok(())
    }

    pub fn claim_token(ctx: Context<ClaimToken>, bump: u8) -> Result<()> {
    let users = ctx.accounts.user_list.user;
    let _owner = ctx.accounts.user.key;
    for (index, _owner) in users.iter().enumerate() {
        let amount = ctx.accounts.user_list.token[index];
        if amount == 0 {
            return Err(ErrorCode::NotEligible.into());
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
            amount as u64,
        )?;
        ctx.accounts.user_list.token[index] = 0;
    }
    Ok(())
}


}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = owner,seeds = ["list".as_ref()] ,bump,space = 8 + 180)]
    pub user_list: Account<'info, User>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateUser<'info> {
    #[account(mut)]
    pub user_list: Account<'info, User>,
    #[account(mut)]
    pub owner: Signer<'info>,
    ///CHECK
    pub user_pubkey: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CheckEligibility<'info> {
    #[account(mut)]
    pub user_list: Account<'info, User>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimToken<'info> {
    #[account(mut)]
    pub user_list: Account<'info, User>,
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
pub struct User {
    user: [Pubkey; 5],
    token: [u32; 5],
}

#[error_code]
pub enum Errors {
    #[msg("you are not eligible!")]
    NotEligible,
}

#[error_code]
pub enum ErrorCode {
    #[msg("you can not claim again!")]
    NotEligible,
}
