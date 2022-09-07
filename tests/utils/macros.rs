#[macro_export]
macro_rules! assert_result {
    ($some:expr) => {
        assert!($some.is_ok(), "{}", $some.unwrap_err())
    }
}