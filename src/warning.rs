use cosmwasm_std::{attr, Attribute, CosmosMsg, Response};

const WARNING: &str = "neptune_warning";

#[macro_export]
macro_rules! warn {
    ($attrs:ident, $variant:expr) => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str { std::any::type_name::<T>() }
        let name = type_name_of(f);
        let msg = format!("{} In function: {}", $variant.str(), &name[..name.len() - 3]);
        $attrs.push(attr("neptune_warning", msg))
    }};
    ($variant:expr) => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str { std::any::type_name::<T>() }
        let name = type_name_of(f);
        let msg = format!("{} In function: {}", $variant.str(), &name[..name.len() - 3]);
        Ok(Response::new().add_attribute("neptune_warning", msg))
    }};
}

pub enum NeptuneWarning<'a> {
    AmountWasZero,
    AmountBelowThreshold,
    InsuffBalance,
    Recursion,
    Generic(&'a str),
}

impl<'a> NeptuneWarning<'a> {
    pub const fn str(self) -> &'a str {
        match self {
            Self::AmountWasZero => "Amount was zero.",
            Self::AmountBelowThreshold => "Amount is below threshold.",
            Self::InsuffBalance => "Insufficient balance. Amount was reduced.",
            Self::Recursion => "Recursion required.",
            Self::Generic(msg) => msg,
        }
    }

    pub fn attr(self) -> Attribute {
        match self {
            Self::AmountWasZero => attr(WARNING, self.str()),
            Self::AmountBelowThreshold => attr(WARNING, self.str()),
            Self::InsuffBalance => attr(WARNING, self.str()),
            Self::Recursion => attr(WARNING, self.str()),
            Self::Generic(..) => attr(WARNING, self.str()),
        }
    }

    pub fn resp(self) -> Response<CosmosMsg> {
        let mut resp = Response::new();
        resp.attributes = vec![self.attr()];
        resp
    }
}
