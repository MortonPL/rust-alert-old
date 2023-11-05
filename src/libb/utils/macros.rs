/// Prints Option's value or None.
#[macro_export]
macro_rules! printoptionln {
    ($string:expr, $val:expr) => {
        if let Some(x) = $val {
            println!($string, x)
        } else {
            println!($string, Option::<()>::None)
        }
    };
}

/// Prints Option's mapped value or None.
#[macro_export]
macro_rules! printoptionmapln {
    ($string:expr, $val:expr, $fun:expr) => {
        if let Some(x) = $val {
            println!($string, $fun(x))
        } else {
            println!($string, Option::<()>::None)
        }
    };
}
