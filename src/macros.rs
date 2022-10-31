//! `macros.rs`

#[macro_export]
macro_rules! try_from_str {
    ($type_name: ident, $var_name: ident) => {
        match $type_name::from_str($var_name) {
            Ok(value) => value,
            Err(err) => {
                let str = format!(
                    "Failed {}::from_str: `{}`, {}",
                    stringify!($type_name),
                    $var_name,
                    err
                );
                return CString::new(str).unwrap().into_raw();
            }
        }
    };
}
