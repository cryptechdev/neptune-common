use crate::error::{CommonError, CommonResult};

pub fn assert_no_multiple_tx(last_tx_height: &mut u64, current_block_height: u64) -> CommonResult<()> {
    if *last_tx_height == current_block_height {
        Err(CommonError::MultipleTx {})
    } else {
        *last_tx_height = current_block_height;
        Ok(())
    }
}
