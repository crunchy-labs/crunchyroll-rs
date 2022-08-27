#[macro_export]
macro_rules! enum_values {
    ($name:ident, $($field:tt = $value:expr),*) => {
        enum_values!{
            $name,
            #[derive()],
            $(
                $field = $value
            ),*
        }
    };
    ($name:ident, #[derive($($derives:tt),*)], $($field:tt = $value:expr),*) => {
        #[derive($($derives),*)]
        pub enum $name {
            $(
                $field
            ),*,
            Custom(String)
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let value = match self {
                    $(
                        $name::$field => $value
                    ),*,
                    $name::Custom(raw) => raw
                };
                write!(f, "{}", value)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                $name::Custom("".to_string())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                match value.as_str() {
                    $(
                        $value => $name::$field
                    ),*,
                    _ => $name::Custom(value)
                }
            }
        }
    };
}