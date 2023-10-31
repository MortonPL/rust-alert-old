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
