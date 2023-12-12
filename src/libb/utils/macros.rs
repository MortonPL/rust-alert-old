/// Prints Option's value or None.
/// /// # Example:
/// ```rs
/// let x = Some(1);
/// printoptionln!("{}", x);  // will print "2"
///
/// let y: Option<i32> = None;
/// printoptionln!("{}", y);  // will print "None"
/// ```
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
/// # Example:
/// ```rs
/// let x = Some(1);
/// printoptionmapln!("{}", x, |x| x*2);  // will print "2"
///
/// let y: Option<i32> = None;
/// printoptionmapln!("{}", y, |y| y*2);  // will print "None"
/// ```
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

/// Initializes a default (zeroed) value for an array type.
/// # Example:
/// ```rs
/// type A = [u8; 12];
/// let a: A = defaultarray!(A);
/// ```
#[macro_export]
macro_rules! defaultarray {
    ($type:ty) => {
        [0u8; std::mem::size_of::<$type>()]
    };
}

/// Unwraps first argument and uses `assert_eq!()`.
#[macro_export]
macro_rules! unwrap_assert {
    ($l:expr, $r:expr) => {
        assert_eq!($l.unwrap(), $r)
    };
}

/// Unwraps first argument and uses `assert_eq!()`.
#[macro_export]
macro_rules! unwrap_ref_assert {
    ($l:expr, $r:expr) => {
        assert_eq!(&$l.unwrap(), $r)
    };
}

/// Creates an entry point for the app.
#[macro_export]
macro_rules! make_app {
    ($cls:ty $(,$arg:ident)*) => {
        static_assertions::assert_impl_all!($cls: clap::Parser);
        fn main() {
            let args = <$cls>::parse();
            match args.command.run($(args.$arg),*) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}.", e);
                    std::process::exit(1);
                }
            };
        }
    }
}
