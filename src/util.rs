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