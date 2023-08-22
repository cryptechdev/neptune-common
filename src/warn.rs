use cosmwasm_std::Attribute;

pub fn warn(msg: impl Into<String>) -> Attribute {
    Attribute::new("neptune_warning", msg)
}
