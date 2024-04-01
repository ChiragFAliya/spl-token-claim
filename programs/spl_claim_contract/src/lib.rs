use std::vec;

use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

declare_id!("9ifhVnUaXKyqeXUfz16f77Q1dfj4JqrJS4b1VSJChGBa");

#[program]
pub mod spl_claim_contract {

    const ADMIN_2: &str = "BFqRiQQA4oTs4zo6rFCA3fFx1RzXkRwpihDEhWSgGB1m";
    const ADMIN: &str = "4rdE7Ub5w5bc9QvFoYLRVdT3B6aLQUiD84hezHW2JEwi";
    use std::str::FromStr;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let authorized_keys = [
    Pubkey::from_str(ADMIN).unwrap(),
    Pubkey::from_str(ADMIN_2).unwrap(),
  ];
  if !authorized_keys.iter().any(|key| key == ctx.accounts.owner.key) {
    return Err(Errors::NotAuthorized.into());
  }
        Ok(())
    }

    pub fn update_users(
        ctx: Context<UpdateUser>,
        amounts: Vec<u64>,
        users: Vec<Pubkey>,
    ) -> Result<()> {
        let authorized_keys = [
    Pubkey::from_str(ADMIN).unwrap(),
    Pubkey::from_str(ADMIN_2).unwrap(),
  ];
  if !authorized_keys.iter().any(|key| key == ctx.accounts.owner.key) {
    return Err(Errors::NotAuthorized.into());
  }
        let mut user_list = ctx.accounts.user_list.load_mut()?;
        if amounts.len() != users.len() {
            return Err(Errors::InvalidInput.into());
        }
        let mut current_index = ctx.accounts.global.total_users;
        for (_, (amount, user)) in amounts.iter().zip(users.iter()).enumerate() {
            user_list.user[current_index as usize] = *user;
            user_list.token[current_index as usize] = *amount;

            current_index += 1;

            ctx.accounts.global.claimable_tokens += *amount;
        }

        ctx.accounts.global.total_users += users.len() as u64;

        Ok(())
    }

    pub fn claim_token(ctx: Context<ClaimToken>, bump: u8, index: u64) -> Result<()> {
        let user_list = &mut ctx.accounts.user_list.load_mut()?;

        // let binding = ctx.accounts.user_list.clone();
        // let mut user_list = &mut binding.load_mut()?;

        // let users = user_list.user;
        // let owner = ctx.accounts.user.key;

        // if *owner != *cur_user {
        //     return Err(Errors::InvalidIndex.into());
        // }

        let amount = user_list.token[index as usize];
        if amount == 0 {
            return Err(Errors::NotEligible.into());
        }

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.global_ata.to_account_info().clone(),
                    to: ctx.accounts.user_ata.to_account_info().clone(),
                    authority: ctx.accounts.global.to_account_info().clone(),
                },
                &[&[b"global", &[bump]]],
            ),
            amount,
        )?;
        user_list.token[index as usize] = 0;

        ctx.accounts.global.claimable_tokens -= amount;
        ctx.accounts.global.claimed_tokens += amount;

        user_list.user[index as usize] = Pubkey::default();

        Ok(())
    }

    pub fn claim_remaining_tokens(
        ctx: Context<ClaimRemainingTokens>,
        bump: u8,
        amount: u64,
    ) -> Result<()> {
        let authorized_keys = [
    Pubkey::from_str(ADMIN).unwrap(),
    Pubkey::from_str(ADMIN_2).unwrap(),
  ];
  if !authorized_keys.iter().any(|key| key == ctx.accounts.owner.key) {
    return Err(Errors::NotAuthorized.into());
  }


        let remaining_amount = ctx.accounts.global_ata.amount;
        msg!("{}", remaining_amount);
        if amount > remaining_amount {
            return Err(Errors::InvalidInput.into());
        }

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.global_ata.to_account_info().clone(),
                    to: ctx.accounts.owner_ata.to_account_info().clone(),
                    authority: ctx.accounts.global.to_account_info().clone(),
                },
                &[&[b"global", &[bump]]],
            ),
            amount,
        )?;

        ctx.accounts.global.claimable_tokens -= amount;

        Ok(())
    }

    pub fn reset_users(ctx: Context<ResetUserList>) -> Result<()> {
        let authorized_keys = [
    Pubkey::from_str(ADMIN).unwrap(),
    Pubkey::from_str(ADMIN_2).unwrap(),
  ];
  if !authorized_keys.iter().any(|key| key == ctx.accounts.owner.key) {
    return Err(Errors::NotAuthorized.into());
  }
    
        let mut user_list = ctx.accounts.user_list.load_mut()?;
        for i in 0..user_list.user.len() {
            user_list.user[i] = Pubkey::default();
            user_list.token[i] = 0;
        }
    
        ctx.accounts.global.claimable_tokens = 0;
        ctx.accounts.global.claimed_tokens = 0;
        ctx.accounts.global.total_users = 0;
    
        let mut current_index = ctx.accounts.global.total_users;
        current_index = 0;
    
        Ok(())
    }
    
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(zero)]
    pub user_list: AccountLoader<'info, User>,
    #[account(init_if_needed,payer=owner,associated_token::mint= mint, associated_token::authority = global)]
    pub global_ata: Box<Account<'info, TokenAccount>>,
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
    pub user_list: AccountLoader<'info, User>,
    #[account(mut)]
    pub global: Account<'info, Global>,
    #[account(mut)]
    pub owner: Signer<'info>,
}



#[derive(Accounts)]
pub struct ClaimToken<'info> {
    #[account(mut)]
    pub user_list: AccountLoader<'info, User>,
    #[account(mut)]
    pub global: Box<Account<'info, Global>>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,
    #[account(init_if_needed,payer=user,associated_token::mint= mint, associated_token::authority = user)]
    pub user_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub global_ata: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    ///CHECK
    pub associated_token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ClaimRemainingTokens<'info> {
    #[account(mut)]
    pub user_list: AccountLoader<'info, User>,
    #[account(mut)]
    pub global: Account<'info, Global>,
    #[account(mut)]
    pub global_ata: Account<'info, TokenAccount>,
    #[account(init_if_needed,payer=owner,associated_token::mint= mint, associated_token::authority = owner)]
    pub owner_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    ///CHECK
    pub associated_token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ResetUserList<'info> {
    #[account(mut)]
    pub user_list: AccountLoader<'info, User>,
    #[account(mut)]
    pub global: Account<'info, Global>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

#[account]
pub struct Global {
    claimable_tokens: u64,
    claimed_tokens: u64,
    total_users: u64,
}

#[account(zero_copy(unsafe))]
#[repr(C)]
pub struct User {
    user: [Pubkey; 10000],
    token: [u64; 10000],
}

#[error_code]
pub enum Errors {
    #[msg("you are not eligible!")]
    NotEligible,
    #[msg("you are not authorized to do this !")]
    NotAuthorized,
    #[msg("Invalid input!")]
    InvalidInput,
    #[msg("Invalid index!")]
    InvalidIndex,
}


