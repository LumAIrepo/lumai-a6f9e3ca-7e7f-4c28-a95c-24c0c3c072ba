// program/src/lib.rs

use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;

declare_id!("your_program_id");

#[program]
pub mod solana_pay_merchant {
    use super::*;

    pub fn initialize_merchant(
        ctx: Context<InitializeMerchant>,
        merchant_name: String,
    ) -> Result<()> {
        let merchant = &mut ctx.accounts.merchant;
        merchant.authority = ctx.accounts.authority.key();
        merchant.name = merchant_name;
        merchant.total_transactions = 0;
        merchant.total_volume = 0;
        Ok(())
    }

    pub fn process_payment(
        ctx: Context<ProcessPayment>,
        amount: u64,
        reference: Pubkey,
    ) -> Result<()> {
        let merchant = &mut ctx.accounts.merchant;
        let payment = &mut ctx.accounts.payment;

        // Transfer SOL from customer to merchant
        let transfer_ix = system_instruction::transfer(
            &ctx.accounts.customer.key(),
            &merchant.authority,
            amount,
        );

        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.customer.to_account_info(),
                ctx.accounts.merchant_authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Update merchant stats
        merchant.total_transactions += 1;
        merchant.total_volume += amount;

        // Record payment details
        payment.merchant = merchant.key();
        payment.customer = ctx.accounts.customer.key();
        payment.amount = amount;
        payment.timestamp = Clock::get()?.unix_timestamp;
        payment.reference = reference;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMerchant<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 64 + 8 + 8,
    )]
    pub merchant: Account<'info, Merchant>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProcessPayment<'info> {
    #[account(mut)]
    pub merchant: Account<'info, Merchant>,
    #[account(
        init,
        payer = customer,
        space = 8 + 32 + 32 + 8 + 8 + 32,
    )]
    pub payment: Account<'info, Payment>,
    #[account(mut)]
    pub customer: Signer<'info>,
    /// CHECK: Safe because we're just using it as a destination
    #[account(mut)]
    pub merchant_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Merchant {
    pub authority: Pubkey,
    pub name: String,
    pub total_transactions: u64,
    pub total_volume: u64,
}

#[account]
pub struct Payment {
    pub merchant: Pubkey,
    pub customer: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    pub reference: Pubkey,
}