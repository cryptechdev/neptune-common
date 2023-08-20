#[cfg(not(feature = "injective"))]
pub type QueryWrapper = cosmwasm_std::Empty;

#[cfg(feature = "injective")]
pub type QueryWrapper = injective_cosmwasm::InjectiveQueryWrapper;
