use cosmwasm_std::{Decimal256, Uint256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::map::IsZeroed;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Default, JsonSchema)]
pub struct Pool {
    pub balance: Uint256,
    pub shares:  Uint256,
}

#[derive(Debug, PartialEq, JsonSchema)]
pub struct PoolMut<'a> {
    pub balance: &'a mut Uint256,
    pub shares:  &'a mut Uint256,
}

impl<'a> PoolMut<'a> {
    pub fn add_shares(self, shares: Uint256, account: &mut PoolAccount) -> AddSharesResponse {
        let pool_balance = self.balance;
        let pool_shares = self.shares;
        let account_principle = &mut account.principle;
        let account_shares = &mut account.shares;

        let shares_to_issue = shares;
        let balance_to_issue = shares_to_issue * Decimal256::from_ratio(*pool_balance, *pool_shares);

        *account_shares += shares_to_issue;
        *account_principle += balance_to_issue;

        *pool_shares += shares_to_issue;
        *pool_balance += balance_to_issue;

        AddSharesResponse { balance_added: balance_to_issue }
    }

    pub fn add_amount(self, amount: Uint256, account: &mut PoolAccount) -> AddAmountResponse {
        let balance_to_issue = amount;

        let pool_balance = self.balance;
        let pool_shares = self.shares;
        let account_principle = &mut account.principle;
        let account_shares = &mut account.shares;

        let shares_to_issue = if *pool_balance == Uint256::zero() {
            amount
        } else {
            *pool_shares * Decimal256::from_ratio(amount, *pool_balance)
        };

        *account_shares += shares_to_issue;
        *account_principle += balance_to_issue;

        *pool_shares += shares_to_issue;
        *pool_balance += balance_to_issue;

        AddAmountResponse { shares_added: shares_to_issue }
    }

    pub fn remove_shares(self, shares: Uint256, account: &mut PoolAccount) -> RemoveSharesResponse {
        let pool_balance = self.balance;
        let pool_shares = self.shares;
        let account_principle = &mut account.principle;
        let account_shares = &mut account.shares;

        let shares_to_remove = if shares > *account_shares {
            *account_shares
        } else {
            shares
        };
        let fraction_to_withdraw = Decimal256::from_ratio(shares_to_remove, *pool_shares);
        let amount_to_remove = *pool_balance * fraction_to_withdraw;

        *account_shares -= shares_to_remove;
        *account_principle = account_principle.saturating_sub(amount_to_remove);

        *pool_shares -= shares_to_remove;
        *pool_balance -= amount_to_remove;

        RemoveSharesResponse { balance_removed: amount_to_remove }
    }

    pub fn remove_amount(self, amount: Uint256, account: &mut PoolAccount) -> RemoveAmountResponse {
        let pool_balance = self.balance;
        let pool_shares = self.shares;
        let account_principle = &mut account.principle;
        let account_shares = &mut account.shares;

        let mut amount_to_remove = amount;
        let mut shares_to_remove = if *pool_balance == Uint256::zero() {
            Uint256::zero()
        } else {
            *pool_shares * Decimal256::from_ratio(amount_to_remove, *pool_balance)
        };

        if shares_to_remove > *account_shares {
            shares_to_remove = *account_shares;
            let fraction_to_withdraw = Decimal256::from_ratio(shares_to_remove, *pool_shares);
            amount_to_remove = *pool_balance * fraction_to_withdraw;
        }

        *account_shares -= shares_to_remove;
        *account_principle = account_principle.saturating_sub(amount_to_remove);

        *pool_shares -= shares_to_remove;
        *pool_balance -= amount_to_remove;

        RemoveAmountResponse { amount_removed: amount_to_remove, shares_removed: shares_to_remove }
    }

    pub fn increase_balance(self, amount: Uint256) {
        let pool_balance = self.balance;
        *pool_balance += amount;
    }

    pub fn decrease_balance(self, amount: Uint256) {
        let pool_balance = self.balance;
        *pool_balance = pool_balance.saturating_sub(amount);
    }

    pub fn get_account_balance(self, account: PoolAccount) -> Uint256 {
        account.shares * Decimal256::from_ratio(*self.balance, *self.shares)
    }
}

impl Pool {
    pub fn new() -> Self { Self { balance: Uint256::zero(), shares: Uint256::zero() } }

    pub fn into_ref(&mut self) -> PoolMut { PoolMut { balance: &mut self.balance, shares: &mut self.shares } }

    pub fn add_shares(&mut self, shares: Uint256, account: &mut PoolAccount) -> AddSharesResponse {
        self.into_ref().add_shares(shares, account)
    }

    pub fn add_amount(&mut self, amount: Uint256, account: &mut PoolAccount) -> AddAmountResponse {
        self.into_ref().add_amount(amount, account)
    }

    pub fn remove_shares(&mut self, shares: Uint256, account: &mut PoolAccount) -> RemoveSharesResponse {
        self.into_ref().remove_shares(shares, account)
    }

    pub fn remove_amount(&mut self, amount: Uint256, account: &mut PoolAccount) -> RemoveAmountResponse {
        self.into_ref().remove_amount(amount, account)
    }

    pub fn increase_balance(&mut self, amount: Uint256) { self.into_ref().increase_balance(amount) }

    pub fn decrease_balance(&mut self, amount: Uint256) { self.into_ref().decrease_balance(amount) }

    pub fn get_account_balance(&self, account: PoolAccount) -> Uint256 {
        account.shares * Decimal256::from_ratio(self.balance, self.shares)
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Default, PartialEq, JsonSchema)]
pub struct PoolAccount {
    pub principle: Uint256,
    pub shares:    Uint256,
}

pub struct AddSharesResponse {
    pub balance_added: Uint256,
}

pub struct AddAmountResponse {
    pub shares_added: Uint256,
}

pub struct RemoveSharesResponse {
    pub balance_removed: Uint256,
}

pub struct RemoveAmountResponse {
    pub amount_removed: Uint256,
    pub shares_removed: Uint256,
}

impl IsZeroed for PoolAccount {
    fn is_zeroed(&self) -> bool { self.shares.is_zero() }
}

mod test {
    #[test]
    fn pool_test() {
        use cosmwasm_std::Uint256;

        use super::*;

        let mut pool: Pool = Pool::default();
        let mut account1: PoolAccount = PoolAccount::default();
        let mut account2: PoolAccount = PoolAccount::default();

        pool.add_amount(Uint256::from(100u64), &mut account1);
        assert_eq!(pool.balance, Uint256::from(100u64));
        assert_eq!(pool.shares, Uint256::from(100u64));
        assert_eq!(account1.principle, Uint256::from(100u64));
        assert_eq!(account1.shares, Uint256::from(100u64));

        pool.increase_balance(Uint256::from(100u64));
        assert_eq!(pool.balance, Uint256::from(200u64));
        assert_eq!(pool.shares, Uint256::from(100u64));
        assert_eq!(account1.principle, Uint256::from(100u64));
        assert_eq!(account1.shares, Uint256::from(100u64));
        assert_eq!(pool.get_account_balance(account1), Uint256::from(200u64));

        pool.add_shares(Uint256::from(50u64), &mut account2);
        assert_eq!(pool.balance, Uint256::from(300u64));
        assert_eq!(pool.shares, Uint256::from(150u64));
        assert_eq!(account2.principle, Uint256::from(100u64));
        assert_eq!(account2.shares, Uint256::from(50u64));

        pool.remove_amount(Uint256::from(100u64), &mut account2);
        assert_eq!(pool.balance, Uint256::from(200u64));
        assert_eq!(pool.shares, Uint256::from(101u64));
        assert_eq!(account2.principle, Uint256::from(0u64));
        assert_eq!(account2.shares, Uint256::from(1u64));

        pool.remove_shares(Uint256::from(100u64), &mut account2);
        assert_eq!(pool.balance, Uint256::from(199u64));
        assert_eq!(pool.shares, Uint256::from(100u64));
        assert_eq!(account2.principle, Uint256::from(0u64));
        assert_eq!(account2.shares, Uint256::from(0u64));
    }
}
