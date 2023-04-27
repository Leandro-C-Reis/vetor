#[macro_export]
macro_rules! log {
    ($($e:expr),+) => {
        {
            #[cfg(debug_assertions)]
            {
                println!($($e),+)
            }
            #[cfg(not(debug_assertions))]
            {
                std::convert::identity($($e),+)
            }
        }
    };
}

#[macro_export]
macro_rules! cstr {
    ($e:expr) => {
        CString::new($e).unwrap()
    }
}