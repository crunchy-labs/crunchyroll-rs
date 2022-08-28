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

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
                where D: serde::Deserializer<'de>
            {
                $crate::internal::serde::string_to_enum(deserializer)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: serde::ser::Serializer {
                serializer.serialize_str(self.to_string().as_str())
            }
        }
    };
}

#[macro_export]
macro_rules! options {
    ($name:ident, $($field:tt($t:ty, $query_name:literal) = $default:expr),*) => {
        #[derive(smart_default::SmartDefault)]
        pub struct $name {
            $(
                #[default($default)]
                $field: Option<$t>
            ),*
        }

        impl $name {
            $(
                pub fn $field(mut self, value: $t) -> $name {
                    self.$field = Some(value);

                    self
                }
            )*

            #[allow(dead_code)]
            pub(crate) fn to_query(&self, extra_params: &[(String, String)]) -> Vec<(String, String)> {
                [
                    extra_params,
                    &[
                        $(
                            ($query_name.to_string(), if let Some(field) = &self.$field {
                                // this workaround is required because serde_urlencoded::to_string
                                // cannot deserialize non map / sequence values.
                                serde_urlencoded::to_string(&[("hack", field)]).unwrap().strip_prefix("hack=").unwrap().to_string()
                            } else {
                                "".to_string()
                            })
                        ),*
                    ]
                ].concat().to_vec()
            }
        }
    }
}
