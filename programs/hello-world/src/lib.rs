use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token};

mod errors;
use errors::ErrorCodes;

declare_id!("6GFDLfsRnPkVHnbmez1CBW8KYAD9xvg2KeKZ7E3zpeDh");

const ALLOW_ADMIN_KEYS: Vec<Pubkey> = vec![];

#[program]
pub mod hello_world {
    use anchor_spl::token::Transfer;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let staking_account = &mut ctx.accounts.staking_account;
        match ALLOW_ADMIN_KEYS
            .iter()
            .position(|&x| x == staking_account.admin)
        {
            Some(_) => {
                staking_account.admin = *ctx.accounts.payer.key; // 设置管理员公钥
                Ok(())
            }
            _ => err!(ErrorCodes::Unauthorized),
        }
    }

    pub fn add_supported_token(ctx: Context<AddSupportedToken>) -> Result<()> {
        let staking_account: &mut Account<StakingAccount> = &mut ctx.accounts.staking_account;

        // 检查调用者是否为管理员
        require!(
            staking_account.admin == *ctx.accounts.payer.key,
            ErrorCodes::Unauthorized
        );

        staking_account
            .supported_tokens
            .push(ctx.accounts.token.token_key);
        Ok(())
    }

    pub fn stake_token(ctx: Context<StakeToken>) -> Result<()> {
        let staking_account: &mut Account<StakingAccount> = &mut ctx.accounts.staking_account;
        let staking = &ctx.accounts.staking;

        require!(
            staking_account.supported_tokens.contains(&staking.token),
            ErrorCodes::UnsupportedToken
        );
        require!(staking.amount > 0, ErrorCodes::InvalidAmount);

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.from_account.to_account_info(),
                    to: ctx.accounts.to_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            staking.amount,
        )?;

        match staking_account
            .user_balances
            .iter()
            .position(|k| k.user_key == ctx.accounts.payer.key())
        {
            Some(i) => {
                let token_balance = &mut staking_account.user_balances[i].token_balance;
                match token_balance
                    .iter()
                    .position(|k| k.token_key == staking.token)
                {
                    Some(j) => {
                        token_balance[j].balance += staking.amount;
                    }
                    _ => token_balance.push(TokenBalance {
                        token_key: staking.token,
                        balance: staking.amount,
                    }),
                }
            }
            _ => staking_account.user_balances.push(UserTokenBalance {
                user_key: ctx.accounts.payer.key(),
                token_balance: vec![TokenBalance {
                    token_key: staking.token,
                    balance: staking.amount,
                }],
            }),
        }
        ctx.accounts.payer.key();
        Ok(())
    }

    pub fn withdraw_token(ctx: Context<WithdrawToken>) -> Result<()> {
        let staking_account: &mut Account<StakingAccount> = &mut ctx.accounts.staking_account;
        let withdraw = &ctx.accounts.withdraw;

        let user_balance = staking_account
            .user_balances
            .iter_mut()
            .find(|b| b.user_key == ctx.accounts.payer.key());
        if let Some(balance) = user_balance {
            let token_balance = balance
                .token_balance
                .iter_mut()
                .find(|b| b.token_key == withdraw.token);
            if let Some(token_bal) = token_balance {
                require!(
                    token_bal.balance >= withdraw.amount,
                    ErrorCodes::ExceedsLimit
                );

                token::transfer(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.from_account.to_account_info(),
                            to: ctx.accounts.to_account.to_account_info(),
                            authority: ctx.accounts.payer.to_account_info(),
                        },
                    ),
                    withdraw.amount,
                )?;
                token_bal.balance -= withdraw.amount;
            } else {
                return err!(ErrorCodes::ExceedsLimit);
            }
        } else {
            return err!(ErrorCodes::ExceedsLimit);
        }

        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct TokenBalance {
    token_key: Pubkey,
    balance: u64,
}

#[account]
#[derive(InitSpace)]
pub struct UserTokenBalance {
    user_key: Pubkey,
    #[max_len(50)]
    token_balance: Vec<TokenBalance>,
}

#[account]
#[derive(InitSpace)]
pub struct StakingAccount {
    pub admin: Pubkey, // 管理员公钥
    #[max_len(50)]
    pub supported_tokens: Vec<Pubkey>,
    #[max_len(500)]
    pub user_balances: Vec<UserTokenBalance>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + StakingAccount::INIT_SPACE)]
    pub staking_account: Account<'info, StakingAccount>,
    #[account(mut)]
    pub payer: Signer<'info>, // 初始化时的签名者即为管理员
    pub system_program: Program<'info, System>,
}

#[account]
pub struct AddToken {
    token_key: Pubkey,
}

#[derive(Accounts)]
pub struct AddSupportedToken<'info> {
    #[account(mut)]
    pub staking_account: Account<'info, StakingAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token: Account<'info, AddToken>,
}

#[account]
pub struct Stake {
    token: Pubkey,
    amount: u64,
}

#[derive(Accounts)]
pub struct StakeToken<'info> {
    #[account(mut)]
    pub staking_account: Account<'info, StakingAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub staking: Account<'info, Stake>,
    #[account(mut)]
    pub from_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, token::Token>,
}

#[account]
pub struct Withdraw {
    token: Pubkey,
    amount: u64,
}

#[derive(Accounts)]
pub struct WithdrawToken<'info> {
    #[account(mut)]
    pub staking_account: Account<'info, StakingAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub withdraw: Account<'info, Withdraw>,
    #[account(mut)]
    pub from_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, token::Token>,
}
