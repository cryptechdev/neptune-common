/// This macro prints debug information to the wasm attributes
/// and then exits without sending any additional messages.
/// Additional information is included in the attributes like the function name.
/// ```
/// # use cosmwasm_std::{attr, Response, StdResult};
/// # use neptune_common::debug;
///     let mut attrs = vec![];
///     let a = (1u32, 2u32, "Hello World".to_string());
///     let b = "data";
///     debug!(attrs, a, b); // attrs will be populated with debug information of a, b, ...
///     let res: Response<()> = Response::default().add_attributes(attrs); // Attributes must be manually added to the response
/// ```
#[macro_export]
macro_rules! debug {
        ($attrs:ident, $( $vars:expr ),*) => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        let func_name = format!("In function: {}", &name[..name.len() - 3]);
        $attrs.push(attr("neptune_debug", func_name));
        $(
            $attrs.push(attr(stringify!($vars), format!("{0:?}",$vars)));
        )*
    }};
}

/// This macro prints debug information to the wasm attributes
/// and then exits without sending any additional messages.
/// Additional information is included in the attributes like the function name.
/// ```
/// # use cosmwasm_std::{attr, Response, StdResult};
/// # use neptune_common::debug_and_exit;
/// # fn test_execute() -> StdResult<Response> {
/// let a = (1u32, 2u32, "Hello World".to_string());
/// let b = "data";
/// debug_and_exit!(a, b); // Function will exit here and print the debug information of a, b, ...
/// Ok(Response::default()) // This line will never be reached
/// # }
/// # test_execute().unwrap();
/// ```
#[macro_export]
macro_rules! debug_and_exit {
    ($( $vars:expr ),*) => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        let func_name = format!("In function: {}", &name[..name.len() - 3]);
        let mut attrs = vec![];
        attrs.push(attr("neptune_debug", func_name));
        $(
            attrs.push(attr(stringify!($vars), format!("{0:?}",$vars)));
        )*
        return Ok(Response::new().add_attributes(attrs));
    }};
}
