/// This module contains the warn! macro.
/// The purpose of this macro is to easily pass warnings as wasm attributes.
/// Additional information is included in the attributes like the function name.
/// Display must be implemented for the expression passed into warn!.
/// ```
/// # use cosmwasm_std::{attr, Response, StdError};
/// # use neptune_common::warn;
/// # fn test_warn() -> Result<Response, StdError> {
/// let mut attrs = vec![warn!("This is a warning")]; // attrs will be populated though Display
/// let another = "This is another warning";
/// warn!(attrs, another); // attrs can be pushed with this syntax.
/// #   Ok(Response::default().add_attributes(attrs))
/// # }
/// ```
#[macro_export]
macro_rules! warn {
    ($attrs:ident, $variant:expr) => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str { std::any::type_name::<T>() }
        let name = type_name_of(f);
        let msg = format!("{} In function: {}", $variant, &name[..name.len() - 3]);
        $attrs.push(cosmwasm_std::attr("neptune_warning", msg))
    }};
    ($variant:expr) => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str { std::any::type_name::<T>() }
        let name = type_name_of(f);
        let msg = format!("{} In function: {}", $variant, &name[..name.len() - 3]);
        attr("neptune_warning", msg)
    }};
}
