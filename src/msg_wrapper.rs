#[cfg(feature = "generic")]
pub type MsgWrapper = cosmwasm_std::Empty;

#[cfg(feature = "injective")]
pub type MsgWrapper = injective_cosmwasm::InjectiveMsgWrapper;
