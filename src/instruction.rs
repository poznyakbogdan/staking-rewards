use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum StakingInstruction {
    ///
    /// 0. [signer] - authority/admin
    /// 1. [writable] - metadata account(pda)
    /// 2. [] - staking token mint account
    /// 3. [] - rewards token mint account
    /// 4. [writable] - escrow staking token account
    /// 5. [writable] - escrow rewards token account
    /// 6. [] - system program
    /// 7. [] - token program
    Init,

    ///
    /// 0. [signer] - user account who want to stake
    /// 1. [writable] - user staking token account 
    /// 2. [writable] - escrow staking token account
    /// 3. [writable] - user state account(pda)
    /// 4. [writable] - metadata account(pda)
    /// 5. [] - staking token mint account
    /// 6. [] - token program
    /// 7. [] - system program
    Stake {
        amount: u64
    },

    ///
    /// 0. [signer] - user account who want to unstake
    /// 1. [writable] - user staking token account 
    /// 2. [writable] - user state account(pda)
    /// 3. [writable] - metadata account(pda)
    /// 4. [writable] - escrow staking token account
    /// 5. [] - escrow staking token owner account(pda)
    /// 6. [] - staking token mint account
    /// 7. [] - token program
    Unstake {
        amount: u64
    },

    ///
    /// 0. [signer] - user account who want to claim rewards
    /// 1. [writable] - user rewards token account 
    /// 2. [writable] - user state account(pda)
    /// 3. [writable] - metadata account(pda)
    /// 4. [writable] - escrow rewards token account
    /// 5. [] - escrow rewards token owner account(pda)
    /// 6. [] - rewards token mint account
    /// 7. [] - token program
    GetRewards
}