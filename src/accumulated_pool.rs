use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal256, DepsMut, Timestamp, Uint256};
use cw_storage_plus::Map;

use crate::{
    error::NeptuneResult,
    pool::{Pool, PoolAccount},
};

/// This data type helps to keep track of pooling together assets between multiple accounts.
#[cw_serde]
pub struct AccumulatedPool {
    pub pool: Pool,
    pub namespace: String,
}

#[cw_serde]
pub struct AccumulatedPoolAccount {
    pub pool_account: PoolAccount,
    pub accumulator: Uint256,
    pub last_accumulation: Option<Timestamp>,
}

impl AccumulatedPool {
    /// Accumulates the pool and updates the account's accumulator.
    /// Should be called before any changes to pool or account balances.
    pub fn accumulate(
        &mut self,
        deps: DepsMut,
        time: Timestamp,
        account: &mut AccumulatedPoolAccount,
    ) -> NeptuneResult<()> {
        let accumulator = Map::<u64, Decimal256>::new(&self.namespace);

        // Calculate pool accumulation
        let last_accumulation = accumulator.last(deps.storage)?;
        let accumulation = match last_accumulation {
            Some(last) => {
                let duration = time.nanos() - last.0;
                if duration == 0 {
                    last.1
                } else {
                    let new_accumulation = Decimal256::from_ratio(
                        self.pool.balance * Uint256::from(duration),
                        self.pool.shares,
                    );
                    let accumulation = last.1 + new_accumulation;
                    accumulator.save(deps.storage, time.nanos(), &accumulation)?;
                    accumulation
                }
            }
            None => {
                accumulator.save(deps.storage, time.nanos(), &Decimal256::zero())?;
                Decimal256::zero()
            }
        };

        // Load the pool's accumulation at the time of the accounts previous accumulation
        if let Some(last) = account.last_accumulation {
            let last_accumulation = accumulator.load(deps.storage, last.nanos())?;
            let account_accumulation_since =
                account.pool_account.shares * (accumulation - last_accumulation);
            account.accumulator += account_accumulation_since;
        }
        account.last_accumulation = Some(time);

        Ok(())
    }
}
