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
