use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_spl::token_interface::{transfer_checked, TransferChecked};
use solana_program::{pubkey, pubkey::Pubkey};

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("FR87ZA1sCVabEMe21X24WkaBqqWdMne18UzGgKdQb4pE");

pub const GAME_PREFIX: &str = "game";
pub const PALYER_PREFIX: &str = "player";
pub const SOLANA_ADDRESS: Pubkey = pubkey!("So11111111111111111111111111111111111111111");
pub const LAMPORT_PER_SOL: u64 = 1000000000;

pub const box_one_price: u64 = (0.1 * LAMPORT_PER_SOL as f64) as u64; // 0.1 sol
pub const box_one_chances: [u64; 5] = [93, 3, 2, 1, 1]; // Percentages
pub const box_one_win_values: [u64; 5] = [
    (0.01 * LAMPORT_PER_SOL as f64) as u64,
    (0.5 * LAMPORT_PER_SOL as f64) as u64,
    LAMPORT_PER_SOL,
    (1.5 * LAMPORT_PER_SOL as f64) as u64,
    (2 * LAMPORT_PER_SOL as f64) as u64,
];

pub const box_two_price: u64 = (0.25 * LAMPORT_PER_SOL as f64) as u64; // 0.1 sol
pub const box_two_chances: [u64; 5] = [70, 12, 10, 7, 1]; // Percentages
pub const box_two_win_values: [u64; 5] = [
    (0.01 * LAMPORT_PER_SOL as f64) as u64,
    (0.25 * LAMPORT_PER_SOL as f64) as u64,
    (0.5 * LAMPORT_PER_SOL as f64) as u64,
    LAMPORT_PER_SOL,
    (5 * LAMPORT_PER_SOL as f64) as u64,
];

pub const box_three_price: u64 = (0.5 * LAMPORT_PER_SOL as f64) as u64; // 0.1 sol
pub const box_three_chances: [u64; 4] = [70, 20, 8, 2]; // Percentages
pub const box_three_win_values: [u64; 4] = [
    (0.25 * LAMPORT_PER_SOL as f64) as u64,
    (0.75 * LAMPORT_PER_SOL as f64) as u64,
    LAMPORT_PER_SOL,
    (2 * LAMPORT_PER_SOL as f64) as u64,
];

pub const box_one_token_price: f64 = 100000.0; // 0.1
pub const box_one_token_chances: [u64; 3] = [87, 8, 5]; // Percentages
pub const box_one_token_win_values: [f64; 3] = [1000.0, 500000.0, 1000000.0];

pub const box_two_token_price: f64 = 50000.0; // 0.2
pub const box_two_token_chances: [u64; 4] = [70, 15, 10, 5]; // Percentages
pub const box_two_token_win_values: [f64; 4] = [1000.0, 100000.0, 150000.0, 200000.0];

pub const box_three_token_price: f64 = 100000.0; // 0.3
pub const box_three_token_chances: [u64; 4] = [65, 25, 9, 1]; // Percentages
pub const box_three_token_win_values: [f64; 4] = [1000.0, 150000.0, 200000.0, 400000.0];

#[program]
mod lottery_game {
    use super::*;
    pub fn init_game(ctx: Context<InitGameCtx>, ix: InitGameIx) -> Result<()> {
        let bump = ctx.bumps.game;

        let new_game = Game {
            bump,
            authority: ctx.accounts.payer.key(),
            default_multiplier: 0,
            token_address: Pubkey::default(),
            created_at: Clock::get().unwrap().unix_timestamp,
        };

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.game.to_account_info(),
            },
        );
        system_program::transfer(cpi_context, ix.amount)?;

        let game = &mut ctx.accounts.game;

        game.set_inner(new_game);
        Ok(())
    }
    pub fn add_token(ctx: Context<AddTokenCtx>, ix: AddTokenIx) -> Result<()> {
        let decimals = 1 * 10u64.pow(ix.default_multiplier.try_into().unwrap());

        let cpi_accounts = Transfer {
            from: ctx.accounts.payer_token_account.to_account_info(),
            to: ctx.accounts.game_token_account.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, ix.amount * decimals)?;

        let game = &mut ctx.accounts.game;
        if game.token_address != Pubkey::default() {
            return err!(ErrorCode::TokenAlreadyAdded);
        }
        game.default_multiplier = ix.default_multiplier;
        game.token_address = ctx.accounts.mint.key();

        Ok(())
    }

    pub fn play_sol(ctx: Context<PlaySolCtx>, ix: PlayIx) -> Result<()> {
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.game.to_account_info(),
            },
        );

        let mut win_amount = 0;

        if ix.box_type == "one" {
            system_program::transfer(cpi_context, box_one_price)?;

            let total_chance: u64 = box_one_chances.iter().sum();
            let mut cumulative_chances = vec![0u64; box_one_chances.len()];
            let mut cumulative_chance: u64 = 0;
            for (i, &chance) in box_one_chances.iter().enumerate() {
                cumulative_chance += chance;
                cumulative_chances[i] = cumulative_chance;
            }

            let clock = Clock::get()?;
            let rand_num = clock.unix_timestamp % 100;

            let mut index = 0;
            for (i, &cumulative_chance) in cumulative_chances.iter().enumerate() {
                if rand_num <= cumulative_chance.try_into().unwrap() {
                    index = i;
                    break;
                }
            }

            let selected_output_value = box_one_win_values[index];
            win_amount = selected_output_value;
        } else if ix.box_type == "two" {
            system_program::transfer(cpi_context, box_two_price)?;

            let total_chance: u64 = box_two_chances.iter().sum();
            let mut cumulative_chances = vec![0u64; box_two_chances.len()];
            let mut cumulative_chance: u64 = 0;
            for (i, &chance) in box_two_chances.iter().enumerate() {
                cumulative_chance += chance;
                cumulative_chances[i] = cumulative_chance;
            }

            let clock = Clock::get()?;
            let rand_num = clock.unix_timestamp % 100;

            let mut index = 0;
            for (i, &cumulative_chance) in cumulative_chances.iter().enumerate() {
                if rand_num <= cumulative_chance.try_into().unwrap() {
                    index = i;
                    break;
                }
            }

            let selected_output_value = box_two_win_values[index];
            win_amount = selected_output_value;
        } else if ix.box_type == "three" {
            system_program::transfer(cpi_context, box_three_price)?;

            let total_chance: u64 = box_three_chances.iter().sum();
            let mut cumulative_chances = vec![0u64; box_three_chances.len()];
            let mut cumulative_chance: u64 = 0;
            for (i, &chance) in box_three_chances.iter().enumerate() {
                cumulative_chance += chance;
                cumulative_chances[i] = cumulative_chance;
            }

            let clock = Clock::get()?;
            let rand_num = clock.unix_timestamp % 100;

            let mut index = 0;
            for (i, &cumulative_chance) in cumulative_chances.iter().enumerate() {
                if rand_num <= cumulative_chance.try_into().unwrap() {
                    index = i;
                    break;
                }
            }

            let selected_output_value = box_three_win_values[index];
            win_amount = selected_output_value;
        }

        let player = &mut ctx.accounts.player;
        player.bump = ctx.bumps.player;
        player.authority = ctx.accounts.payer.key();
        player.identifier = ix.identifier;
        player.win_amount = win_amount;
        player.token_type = "sol".to_string();
        player.claimed = false;
        player.created_at = Clock::get().unwrap().unix_timestamp;

        Ok(())
    }

    pub fn play_token(ctx: Context<PlayTokenCtx>, ix: PlayIx) -> Result<()> {
        let decimals = 1 * 10u64.pow(ctx.accounts.game.default_multiplier.try_into().unwrap());

        let cpi_accounts = Transfer {
            from: ctx.accounts.payer_token_account.to_account_info(),
            to: ctx.accounts.game_token_account.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        let mut win_amount = 0.0;

        if ix.box_type == "one" {
            let price: u64 = (box_one_token_price * decimals as f64) as u64;
            token::transfer(cpi_ctx, price)?;

            let total_chance: u64 = box_one_token_chances.iter().sum();
            let mut cumulative_chances = vec![0u64; box_one_token_chances.len()];
            let mut cumulative_chance: u64 = 0;
            for (i, &chance) in box_one_token_chances.iter().enumerate() {
                cumulative_chance += chance;
                cumulative_chances[i] = cumulative_chance;
            }

            let clock = Clock::get()?;
            let rand_num = clock.unix_timestamp % 100;

            let mut index = 0;
            for (i, &cumulative_chance) in cumulative_chances.iter().enumerate() {
                if rand_num <= cumulative_chance.try_into().unwrap() {
                    index = i;
                    break;
                }
            }

            let selected_output_value = box_one_token_win_values[index];
            win_amount = selected_output_value;
        } else if ix.box_type == "two" {
            let price: u64 = (box_two_token_price * decimals as f64) as u64;
            token::transfer(cpi_ctx, price)?;

            let total_chance: u64 = box_two_token_chances.iter().sum();
            let mut cumulative_chances = vec![0u64; box_two_token_chances.len()];
            let mut cumulative_chance: u64 = 0;
            for (i, &chance) in box_two_token_chances.iter().enumerate() {
                cumulative_chance += chance;
                cumulative_chances[i] = cumulative_chance;
            }

            let clock = Clock::get()?;
            let rand_num = clock.unix_timestamp % 100;

            let mut index = 0;
            for (i, &cumulative_chance) in cumulative_chances.iter().enumerate() {
                if rand_num <= cumulative_chance.try_into().unwrap() {
                    index = i;
                    break;
                }
            }

            let selected_output_value = box_two_token_win_values[index];
            win_amount = selected_output_value;
        } else if ix.box_type == "three" {
            let price: u64 = (box_three_token_price * decimals as f64) as u64;
            token::transfer(cpi_ctx, price)?;

            let total_chance: u64 = box_three_token_chances.iter().sum();
            let mut cumulative_chances = vec![0u64; box_three_token_chances.len()];
            let mut cumulative_chance: u64 = 0;
            for (i, &chance) in box_three_token_chances.iter().enumerate() {
                cumulative_chance += chance;
                cumulative_chances[i] = cumulative_chance;
            }

            let clock = Clock::get()?;
            let rand_num = clock.unix_timestamp % 100;

            let mut index = 0;
            for (i, &cumulative_chance) in cumulative_chances.iter().enumerate() {
                if rand_num <= cumulative_chance.try_into().unwrap() {
                    index = i;
                    break;
                }
            }

            let selected_output_value = box_three_token_win_values[index];
            win_amount = selected_output_value;
        }

        let player = &mut ctx.accounts.player;
        player.bump = ctx.bumps.player;
        player.authority = ctx.accounts.payer.key();
        player.identifier = ix.identifier;
        let win_amount_integer = (win_amount * decimals as f64) as u64;
        player.win_amount = win_amount_integer;
        player.token_type = "token".to_string();
        player.claimed = false;
        player.created_at = Clock::get().unwrap().unix_timestamp;

        Ok(())
    }

    pub fn claim_reward_sol(ctx: Context<ClaimRewardSolCtx>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        ctx.accounts.game.sub_lamports(player.win_amount)?;
        ctx.accounts.payer.add_lamports(player.win_amount)?;

        player.claimed = true;
        player.win_amount = 0;

        Ok(())
    }

    pub fn claim_reward_token(ctx: Context<ClaimRewardTokenCtx>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        let game = &ctx.accounts.game;

        let admin = ctx.accounts.game.authority;

        let game_seeds = &[
            GAME_PREFIX.as_bytes(),
            admin.as_ref(),
            &[ctx.accounts.game.bump],
        ];

        let game_signer_seeds = &[&game_seeds[..]];

        let accounts = TransferChecked {
            from: ctx.accounts.game_token_account.to_account_info(),
            to: ctx.accounts.payer_token_account.to_account_info(),
            authority: ctx.accounts.game.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
        };

        let tx_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            accounts,
            game_signer_seeds,
        );

        transfer_checked(tx_ctx, player.win_amount, game.default_multiplier)?;

        player.claimed = true;
        player.win_amount = 0;

        Ok(())
    }

    pub fn withdraw_sol(ctx: Context<WithdrawSolsCtx>, ix: WithdrawIx) -> Result<()> {
        ctx.accounts.game.sub_lamports(ix.amount)?;
        ctx.accounts.payer.add_lamports(ix.amount)?;

        Ok(())
    }

    pub fn withdraw_token(ctx: Context<TokenWithdrawCtx>, ix: WithdrawIx) -> Result<()> {
        let decimals = 1 * 10u64.pow(ctx.accounts.game.default_multiplier.try_into().unwrap());
        let admin = ctx.accounts.game.authority;

        let game_seeds = &[
            GAME_PREFIX.as_bytes(),
            admin.as_ref(),
            &[ctx.accounts.game.bump],
        ];

        let game_signer_seeds = &[&game_seeds[..]];

        let accounts = TransferChecked {
            from: ctx.accounts.game_token_account.to_account_info(),
            to: ctx.accounts.payer_token_account.to_account_info(),
            authority: ctx.accounts.game.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
        };

        let tx_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            accounts,
            game_signer_seeds,
        );

        msg!("decimals : {}, amount :{}", decimals, ix.amount);

        transfer_checked(
            tx_ctx,
            ix.amount * decimals / 100,
            ctx.accounts.game.default_multiplier,
        )?;

        Ok(())
    }

}

#[derive(Accounts)]
pub struct InitGameCtx<'info> {
    #[account(
        init,
        payer = payer,
        space = 160,
        seeds = [GAME_PREFIX.as_bytes(), payer.key().as_ref()],
        bump
    )]
    game: Account<'info, Game>,
    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddTokenCtx<'info> {
    #[account(mut, constraint = game.authority == payer.key() @ ErrorCode::InvalidAdmin)]
    game: Account<'info, Game>,
    #[account(
        init_if_needed,
        payer = payer, 
        associated_token::mint = mint, 
        associated_token::authority = game
    )]
    game_token_account: Account<'info, TokenAccount>,
    mint: Account<'info, Mint>,

    #[account(mut)]
    payer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    payer: Signer<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    associated_token_program: Program<'info, AssociatedToken>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(ix: PlayIx)]
pub struct PlaySolCtx<'info> {
    #[account(init,
        payer=payer,
        space=100,
        seeds = [PALYER_PREFIX.as_bytes(), ix.identifier.as_ref()],
        bump
        )]
    player: Account<'info, Player>,
    #[account(mut)]
    game: Account<'info, Game>,
    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(ix: PlayIx)]
pub struct PlayTokenCtx<'info> {
    #[account(init,
        payer=payer,
        space=100,
        seeds = [PALYER_PREFIX.as_bytes(), ix.identifier.as_ref()],
        bump)]
    player: Account<'info, Player>,
    #[account(mut)]
    game: Account<'info, Game>,
    #[account(mut)]
    game_token_account: Account<'info, TokenAccount>,
    mint: Account<'info, Mint>,
    #[account(mut)]
    payer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    payer: Signer<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimRewardSolCtx<'info> {
    #[account(mut)]
    player: Account<'info, Player>,
    #[account(mut)]
    game: Account<'info, Game>,
    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimRewardTokenCtx<'info> {
    #[account(mut)]
    player: Account<'info, Player>,
    #[account(mut)]
    game: Account<'info, Game>,
    #[account(mut)]
    game_token_account: Account<'info, TokenAccount>,
    mint: Account<'info, Mint>,
    #[account(mut)]
    payer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    payer: Signer<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawSolsCtx<'info> {
    #[account(mut, constraint = game.authority == payer.key() @ ErrorCode::InvalidAdmin)]
    game: Account<'info, Game>,
    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TokenWithdrawCtx<'info> {
    #[account(mut, constraint = game.authority == payer.key() @ ErrorCode::InvalidAdmin)]
    game: Account<'info, Game>,
    #[account(mut)]
    game_token_account: Account<'info, TokenAccount>,
    mint: Account<'info, Mint>,
    #[account(mut)]
    payer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    payer: Signer<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

#[account]
pub struct Game {
    pub bump: u8,
    pub authority: Pubkey,
    pub token_address: Pubkey,
    pub default_multiplier: u8,
    pub created_at: i64,
}

#[account]
pub struct Player {
    pub bump: u8,
    pub authority: Pubkey,
    pub claimed: bool,
    pub created_at: i64,
    pub win_amount: u64,
    pub token_type: String,
    pub identifier: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitGameIx {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddTokenIx {
    pub default_multiplier: u8,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawIx {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PlayIx {
    pub box_type: String,
    pub identifier: String,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid Admin")]
    InvalidAdmin,
    #[msg("Token Already Added")]
    TokenAlreadyAdded,
}
