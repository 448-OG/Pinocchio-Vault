use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::create_program_address,
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;

use crate::SEED;

pub struct WithdrawAccounts<'a> {
    pub owner: &'a AccountInfo,
    pub vault: &'a AccountInfo,
}

// Perform sanity checks on the accounts
impl<'a> TryFrom<(u8, &'a [AccountInfo])> for WithdrawAccounts<'a> {
    type Error = ProgramError;

    fn try_from(bump_and_accounts: (u8, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let (bump, accounts) = bump_and_accounts;

        let [owner, vault, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Basic Accounts Checks
        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if !vault.is_owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let vault_key = create_program_address(&[SEED, owner.key().as_ref(), &[bump]], &crate::ID)?;
        if &vault_key != vault.key() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Self { owner, vault })
    }
}

pub struct Withdraw<'a> {
    pub accounts: WithdrawAccounts<'a>,
    pub bump: u8,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Withdraw<'a> {
    type Error = ProgramError;

    fn try_from(data_and_accounts: (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let (data, accounts) = data_and_accounts;

        if data.len() != 1 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let bump = *data.first().ok_or(ProgramError::InvalidInstructionData)?;

        let accounts = WithdrawAccounts::try_from((bump, accounts))?;

        Ok(Self { accounts, bump })
    }
}

impl<'a> Withdraw<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        // Create signer seeds for our CPI
        let bump = &[self.bump];
        let seeds = [
            Seed::from(b"vault"),
            Seed::from(self.accounts.owner.key().as_ref()),
            Seed::from(bump),
        ];
        let signers = [Signer::from(&seeds)];

        Transfer {
            from: self.accounts.vault,
            to: self.accounts.owner,
            lamports: self.accounts.vault.lamports(),
        }
        .invoke_signed(&signers)?;

        Ok(())
    }
}
