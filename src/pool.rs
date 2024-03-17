use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint256;

use crate::traits::Zeroed;

/// This data type helps to keep track of pooling together assets between multiple accounts.
#[cw_serde]
#[derive(Copy, Default)]
pub struct Pool {
    pub balance: Uint256,
    pub shares: Uint256,
}

impl GetPoolMut for Pool {
    fn get_pool_mut(&mut self) -> PoolMut {
        PoolMut {
            balance: &mut self.balance,
            shares: &mut self.shares,
        }
    }
}

impl GetPoolRef for Pool {
    fn get_pool_ref(&self) -> PoolRef {
        PoolRef {
            balance: &self.balance,
            shares: &self.shares,
        }
    }
}

/// This serves the same purpose as Pool, but can be constructed directly from immutable references.
pub struct PoolRef<'a> {
    pub balance: &'a Uint256,
    pub shares: &'a Uint256,
}

// This serves the same purpose as Pool, but can be constructed directly from mutable references.
pub struct PoolMut<'a> {
    pub balance: &'a mut Uint256,
    pub shares: &'a mut Uint256,
}

impl GetPoolMut for PoolMut<'_> {
    fn get_pool_mut(&mut self) -> PoolMut {
        PoolMut {
            balance: self.balance,
            shares: self.shares,
        }
    }
}

impl GetPoolRef for PoolMut<'_> {
    fn get_pool_ref(&self) -> PoolRef {
        PoolRef {
            balance: self.balance,
            shares: self.shares,
        }
    }
}

pub trait GetPoolMut {
    fn get_pool_mut(&mut self) -> PoolMut;
}

pub trait GetPoolRef {
    fn get_pool_ref(&self) -> PoolRef;
}

/// Adds shares to an account and calculates the corresponding balance.
pub fn add_shares(
    pool: &mut dyn GetPoolMut,
    shares: Uint256,
    account: &mut PoolAccount,
) -> AddSharesResponse {
    let pool_mut = pool.get_pool_mut();
    let pool_balance = pool_mut.balance;
    let pool_shares = pool_mut.shares;
    let account_principal = &mut account.principal;
    let account_shares = &mut account.shares;

    let shares_to_issue = shares;
    let balance_to_issue = shares_to_issue.multiply_ratio(*pool_balance, *pool_shares);

    *account_shares += shares_to_issue;
    *account_principal += balance_to_issue;

    *pool_shares += shares_to_issue;
    *pool_balance += balance_to_issue;

    AddSharesResponse {
        balance_added: balance_to_issue,
    }
}

/// Adds a balance to an account and calculates the corresponding shares to issue.
pub fn add_amount(
    pool: &mut dyn GetPoolMut,
    amount: Uint256,
    account: &mut PoolAccount,
) -> AddAmountResponse {
    let balance_to_issue = amount;

    let pool_mut = pool.get_pool_mut();
    let pool_balance = pool_mut.balance;
    let pool_shares = pool_mut.shares;
    let account_principal = &mut account.principal;
    let account_shares = &mut account.shares;

    let shares_to_issue = if pool_balance.is_zero() {
        amount
    } else {
        amount.multiply_ratio(*pool_shares, *pool_balance)
    };

    *account_shares += shares_to_issue;
    *account_principal += balance_to_issue;

    *pool_shares += shares_to_issue;
    *pool_balance += balance_to_issue;

    AddAmountResponse {
        shares_added: shares_to_issue,
    }
}

/// Removes shares from an account and calculates the corresponding balance to return.
pub fn remove_shares(
    pool: &mut dyn GetPoolMut,
    shares: Uint256,
    account: &mut PoolAccount,
) -> RemoveSharesResponse {
    let pool_mut = pool.get_pool_mut();
    let pool_balance = pool_mut.balance;
    let pool_shares = pool_mut.shares;
    let account_principal = &mut account.principal;
    let account_shares = &mut account.shares;

    let shares_to_remove = if shares > *account_shares {
        *account_shares
    } else {
        shares
    };

    let amount_to_remove = shares_to_remove.multiply_ratio(*pool_balance, *pool_shares);

    *account_shares -= shares_to_remove;
    *account_principal = account_principal.saturating_sub(amount_to_remove);

    *pool_shares -= shares_to_remove;
    *pool_balance -= amount_to_remove;

    RemoveSharesResponse {
        balance_removed: amount_to_remove,
    }
}

/// Removes a balance from an account and calculates the corresponding shares to return.
pub fn remove_amount(
    pool: &mut dyn GetPoolMut,
    amount: Uint256,
    account: &mut PoolAccount,
) -> RemoveAmountResponse {
    let pool_mut = pool.get_pool_mut();
    let pool_balance = pool_mut.balance;
    let pool_shares = pool_mut.shares;
    let account_principal = &mut account.principal;
    let account_shares = &mut account.shares;

    if pool_balance.is_zero() || pool_shares.is_zero() || account_shares.is_zero() {
        return RemoveAmountResponse {
            amount_removed: Uint256::zero(),
            shares_removed: Uint256::zero(),
        };
    }

    let amount_to_remove;
    let shares_to_remove;
    let account_amount = account_shares.multiply_ratio(*pool_balance, *pool_shares);
    if amount > account_amount {
        amount_to_remove = account_amount;
        shares_to_remove = *account_shares;
    } else {
        amount_to_remove = amount;
        shares_to_remove = account_shares.multiply_ratio(amount, account_amount);
    }

    *account_shares -= shares_to_remove;
    *account_principal = account_principal.saturating_sub(amount_to_remove);

    *pool_shares -= shares_to_remove;
    *pool_balance -= amount_to_remove;

    RemoveAmountResponse {
        amount_removed: amount_to_remove,
        shares_removed: shares_to_remove,
    }
}

/// Increases the balance of the pool by the amount specified.
pub fn increase_balance(pool: &mut dyn GetPoolMut, amount: Uint256) {
    let pool_mut = pool.get_pool_mut();
    let pool_balance = pool_mut.balance;
    *pool_balance += amount;
}

/// Decreases the balance of the pool by the amount specified.
pub fn decrease_balance(pool: &mut dyn GetPoolMut, amount: Uint256) {
    let pool_mut = pool.get_pool_mut();
    let pool_balance = pool_mut.balance;
    *pool_balance = pool_balance.saturating_sub(amount);
}

/// Returns the balance of an account
pub fn get_account_balance(pool: &dyn GetPoolRef, account: PoolAccount) -> Uint256 {
    let pool_ref = pool.get_pool_ref();
    let pool_balance = pool_ref.balance;
    let pool_shares = pool_ref.shares;
    account
        .shares
        .checked_multiply_ratio(*pool_balance, *pool_shares)
        .unwrap_or_default()
}

#[cw_serde]
#[derive(Copy, Default)]
pub struct PoolAccount {
    pub principal: Uint256,
    pub shares: Uint256,
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

impl Zeroed for PoolAccount {
    fn is_zeroed(&self) -> bool {
        self.shares.is_zero()
    }

    fn remove_zeroed(&mut self) {}
}

#[cfg(test)]
mod tests {
    use rand::random;

    use super::*;

    #[test]
    fn test_add_and_remove() {
        for _ in 0..1000 {
            let start_pool_balance = Uint256::from(random::<u64>());
            let start_pool_shares = Uint256::from(random::<u64>());
            let amount = Uint256::from(random::<u64>());

            let mut account = PoolAccount::default();
            let mut pool = Pool {
                balance: start_pool_balance,
                shares: start_pool_shares,
            };
            // add_amount(&mut pool, amount, &mut account);
            //add_amount(&mut pool, amount, &mut account);
            add_amount(&mut pool, amount, &mut account);
            // pool_mut.add_amount(amount, &mut account);
            pool.balance += Uint256::from(random::<u64>());
            let balance = get_account_balance(&pool, account);
            let amount_removed = remove_amount(&mut pool, balance, &mut account);

            assert_eq!(amount_removed.amount_removed, balance);
            assert_eq!(
                get_account_balance(&pool, account),
                Uint256::zero(),
                "start_pool_balance: {start_pool_balance}, start_pool_shares:
{start_pool_shares}, amount: {amount}, account {account:#?}"
            );
        }
    }

    #[test]
    fn pool_test() {
        let mut pool: Pool = Pool::default();
        let mut account1: PoolAccount = PoolAccount::default();
        let mut account2: PoolAccount = PoolAccount::default();

        add_amount(&mut pool, Uint256::from(100u64), &mut account1);
        assert_eq!(pool.balance, Uint256::from(100u64));
        assert_eq!(pool.shares, Uint256::from(100u64));
        assert_eq!(account1.principal, Uint256::from(100u64));
        assert_eq!(account1.shares, Uint256::from(100u64));

        increase_balance(&mut pool, Uint256::from(100u64));
        assert_eq!(pool.balance, Uint256::from(200u64));
        assert_eq!(pool.shares, Uint256::from(100u64));
        assert_eq!(account1.principal, Uint256::from(100u64));
        assert_eq!(account1.shares, Uint256::from(100u64));
        assert_eq!(get_account_balance(&pool, account1), Uint256::from(200u64));

        add_shares(&mut pool, Uint256::from(50u64), &mut account2);
        assert_eq!(pool.balance, Uint256::from(300u64));
        assert_eq!(pool.shares, Uint256::from(150u64));
        assert_eq!(account2.principal, Uint256::from(100u64));
        assert_eq!(account2.shares, Uint256::from(50u64));

        remove_amount(&mut pool, Uint256::from(100u64), &mut account2);
        assert_eq!(pool.balance, Uint256::from(200u64));
        assert_eq!(pool.shares, Uint256::from(100u64));
        assert_eq!(account2.principal, Uint256::from(0u64));
        assert_eq!(account2.shares, Uint256::from(0u64));

        remove_shares(&mut pool, Uint256::from(100u64), &mut account2);
        assert_eq!(pool.balance, Uint256::from(200u64));
        assert_eq!(pool.shares, Uint256::from(100u64));
        assert_eq!(account2.principal, Uint256::from(0u64));
        assert_eq!(account2.shares, Uint256::from(0u64));
    }
}
