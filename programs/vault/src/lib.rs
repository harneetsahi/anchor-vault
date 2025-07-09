#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};


declare_id!("J6GPi9FPnuN5VpxraCF9yHMYfRLw4Ap3UqGK9wT3qUvw");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    pub fn deposit(ctx: Context<Payments>, amount:u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withraw(ctx: Context<Payments>, amount: u64) -> Result <()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<CloseAccounts>) -> Result<()> {
        ctx.accounts.close()
    }
}


#[derive(Accounts)]
pub struct Initialize<'info> {

    #[account(mut)]
    pub signer: Signer<'info>, // verfies the account that signed the transaction

    #[account(
        init,
        payer = signer,
        seeds = [b"state", signer.key().as_ref()], // linking to the signer
        bump,
        space = 8 + VaultState::INIT_SPACE
    )]
    pub vault_state: Account<'info, VaultState>,  // it will be owned by the current program and is used to store the bumps

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()], // linking to the vault_state
        bump
    )] 
    pub vault: SystemAccount<'info>, // confirms the ownership of the account by the system program. SystemAccount is only for holding sol and no other data. And it is initialized automatically by transferring enough lamports

    pub system_program: Program<'info, System> // ensures the account is executable and matches the system program ID, enabling CPIs calls like account creation and lamport transfers.
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result <()> {

        let rent_exempt = Rent::get()?.minimum_balance(self.vault.to_account_info().data_len()); // amt to be transferred to the vault to make it rent exempt

        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.vault.to_account_info()
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);        

        transfer(cpi_ctx, rent_exempt)?;

        self.vault_state.vault_bump = bumps.vault;
        self.vault_state.state_bump = bumps.vault_state;

        Ok(())
    }
} 

#[derive(Accounts)]
pub struct Payments<'info> {
    #[account(mut)]
    pub signer: Signer <'info>,

    #[account(
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault : SystemAccount<'info>,

    pub system_program: Program<'info, System> // this is what allows cpi calls like transferring lamports
}

impl<'info> Payments<'info> {

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.vault.to_account_info()
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, amount)?; 

        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {

        // check that this withdraw function leaves the vault with enough rent-exempt balance

        // check that the account has enough funds for the user to withdraw

        let cpi_program = self.system_program.to_account_info(); // systemprogram is the program that handles transfers


         let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.signer.to_account_info()
        };

        let vault_seeds = &[
            b"vault".as_ref(),
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump]
        ]; // seeds for the program to sign for the PDA

        let signer_seeds = &[&vault_seeds[..]]; // a reference to an outer array where each element is a reference to another array, which represents a full set of seeds for one PDA signer, and inside that middle araay are the individual seeds

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, amount)?; 

        Ok(())
        
    }
}

#[derive(Accounts)]
pub struct CloseAccounts<'info> {

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump,
        close = signer // this is where the rent goes after closing
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>
}

impl<'info> CloseAccounts<'info> {

    pub fn close(&mut self) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.signer.to_account_info()
        };

        let vault_seeds = &[
            b"vault".as_ref(),
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump]
        ];

        let signer_seeds = &[&vault_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, self.vault.lamports())
    }
}

#[account]
#[derive(InitSpace)]
pub struct VaultState { 
    pub vault_bump: u8, // bump for the vault pda
    pub state_bump: u8, // bump for the vaultstate pda
}
