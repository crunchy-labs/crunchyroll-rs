/// This proc-macro allows to crete enums with string values. The syntax for this is like specifying a
/// enum with numerics values, just with strings instead of numbers.
/// Every created enum has a `Custom(String)`
/// field which can be used to represent custom values of the enums purpose (in case the enum
/// holds some static variables, like [`crunchyroll_rs::categories::Category`], and Crunchyroll
/// decides to add a additional variable) which reduces the chance of breaking something.
///
/// The generated enum implements [`std::fmt::Display`] (for a representation of the values),
/// [`Default`] (which is `<name>::Custom("")`), [`From<String>`] (checks if the given string
/// matches a value representation; if not `<name>::Custom("")`) and [`serde::Serialize`] as well
/// as [`serde::Deserialize`] for http actions.
macro_rules! enum_values {
    ($(#[$attribute:meta])* $v:vis enum $name:ident { $($field:ident = $value:expr)* }) => {
        $(
            #[$attribute]
        )*
        #[derive(Clone, Debug, Eq, PartialEq)]
        $v enum $name {
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
                for (v, f) in [$(($value, $name::$field)),*] {
                    if value.eq_ignore_ascii_case(v) {
                        return f;
                    }
                }
                $name::Custom(value)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
                where D: serde::Deserializer<'de>
            {
                Ok(Self::from(String::deserialize(deserializer)?))
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: serde::ser::Serializer {
                serializer.serialize_str(self.to_string().as_str())
            }
        }
    };
}

/// This proc-macro creates a struct which is internal primarily used to specify request options for
/// specific endpoints.
///
/// # Examples
///
/// ```
/// use crunchyroll_rs::options;
///
/// options! {
///     PaginationOptions;
///     limit(u32, "n") = Some(20)
///     start(u32, "start") = None
/// }
/// ```
///
/// Produces the following struct implementation.
///
/// ```
/// pub struct PaginationOptions {
///     limit: Option<u32>,
///     start: Option<u32>
/// }
///
/// impl Default for PaginationOptions {
///     fn default() -> Self {
///         Self {
///             limit: Some(20),
///             start: None
///         }
///     }
/// }
///
/// impl PaginationOptions {
///     pub fn limit(mut self, value: u32) -> PaginationOptions {
///         self.limit = Some(value);
///         self
///     }
///
///     pub fn start(mut self, value: u32) -> PaginationOptions {
///         self.start = Some(value);
///         self
///     }
///
///     pub(crate) fn to_query(&self, extra_params: &[(String, String)]) -> Vec<(String, String)> {
///         [
///             extra_params,
///             &[
///                 // this `unwrap` code does not exactly gets generated, it checks if the value
///                 // is `Some` or `None`, but to show it in a simple way `.unwrap()` should be ok
///                 ("n", self.limit.unwrap()),
///                 ("start", self.start.unwrap())
///             ]
///         ].concat().to_vec()
///     }
/// }
/// ```
macro_rules! options {
    ($name:ident; $($(#[$attribute:meta])* $field:ident($t:ty, $query_name:literal) = $default:expr),*) => {
        #[derive(Clone, Debug)]
        pub struct $name {
            $(
                $(
                    #[$attribute]
                )*
                $field: Option<$t>
            ),*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $(
                        $field: $default
                    ),*
                }
            }
        }

        impl $name {
            $(
                pub fn $field(mut self, value: $t) -> $name {
                    self.$field = Some(value);

                    self
                }
            )*

            #[allow(dead_code)]
            pub(crate) fn to_query(&self) -> Vec<(String, String)> {
                let encoded = serde_urlencoded::to_string([
                    $(
                        ($query_name, if let Some(field) = &self.$field {
                            Some(serde_json::to_value(field).unwrap())
                        } else { None })
                    ),*
                ]).unwrap();
                serde_urlencoded::from_str(encoded.as_str()).unwrap()
            }
        }
    }
}

pub(crate) use enum_values;
pub(crate) use options;
