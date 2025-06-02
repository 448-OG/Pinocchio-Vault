use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::create_program_address,
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;

use crate::DepositInstructionData;

pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub instruction_datas: DepositInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Deposit<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let bump = data.first().ok_or(ProgramError::InvalidAccountData)?;
        let accounts = DepositAccounts::try_from((*bump, accounts))?;

        let instruction_datas: DepositInstructionData =
            DepositInstructionData::try_from(&data[1..])?;

        Ok(Self {
            accounts,
            instruction_datas,
        })
    }
}

impl<'a> Deposit<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&mut self) -> ProgramResult {
        Transfer {
            from: self.accounts.owner,
            to: self.accounts.vault,
            lamports: self.instruction_datas.amount,
        }
        .invoke()?;

        Ok(())
    }
}

pub struct DepositAccounts<'a> {
    pub owner: &'a AccountInfo,
    pub vault: &'a AccountInfo,
}

impl<'a> TryFrom<(u8, &'a [AccountInfo])> for DepositAccounts<'a> {
    type Error = ProgramError;

    fn try_from(bump_and_accounts: (u8, &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let (bump, accounts) = bump_and_accounts;

        let [owner, vault, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Accounts Checks
        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if !vault.is_owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if vault.lamports().ne(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        let vault_key = create_program_address(&[crate::SEED, owner.key(), &[bump]], &crate::ID)?;

        if vault.key().ne(&vault_key) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Return the accounts
        Ok(Self { owner, vault })
    }
}
