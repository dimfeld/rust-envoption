use std::env;
use std::str::FromStr;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;

/// The behavior if the environment variable is not set.
pub enum OptionType<T> {
    /// Return `None`
    Optional,
    /// Return an error
    Required,
    /// Use the default value
    Default(T),
}

#[derive(Debug,PartialEq)]
pub enum EnvOptionError<T> where T: Error {
    /// An error occurred while parsing the environment variable.
    /// This error contains the T::Err type from the `parse` function.
    ParseError(String, T),
    /// The environment variable was missing.
    Missing(String),
}

impl<T> fmt::Display for EnvOptionError<T> where T: Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EnvOptionError::ParseError(ref s, ref err) => write!(f, "parsing {}: {}", s, err),
            EnvOptionError::Missing(ref s) => write!(f, "{} not found", s),
        }
    }
}

impl<T> Error for EnvOptionError<T> where T: Error {
    fn description(&self) -> &str {
        match *self {
            EnvOptionError::ParseError(_, _) => "parse error",
            EnvOptionError::Missing(_) => "variable is required"
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            EnvOptionError::ParseError(_, ref e) => Some(e),
            EnvOptionError::Missing(_) => None
        }
    }
}

/// Get an environment variable, using the given mode to determine behavior when it is not set.
pub fn get<T,B>(var_name: &str, mode: OptionType<B>) -> Result<Option<T>, EnvOptionError<T::Err>>  where B: Into<T>, T : FromStr + Debug, T::Err: Error {
    match env::var(var_name) {
        Err(_) => match mode {
            OptionType::Optional => Ok(None),
            OptionType::Required => Err(EnvOptionError::Missing(var_name.to_string())),
            OptionType::Default(d) => Ok(Some(d.into())),
        },
        Ok(value) => value.parse::<T>().map(|value| Some(value)).map_err(|e| EnvOptionError::ParseError(var_name.to_string(), e)),
    }
}

/// Sugar around get to avoid the extra `Option` when it will never be `None` anyway.
pub fn require<T>(var_name: &str) -> Result<T, EnvOptionError<T::Err>> where T: FromStr + Debug, T::Err: Error {
    get::<T,T>(var_name, OptionType::Required).map(|o| o.unwrap())
}

pub fn with_default<T, B>(var_name: &str, default: B) -> Result<T, EnvOptionError<T::Err>> where B: Into<T>, T: FromStr + Debug, T::Err: Error {
    get(var_name, OptionType::Default(default)).map(|o| o.unwrap())
}

pub fn optional<T>(var_name: &str) -> Result<Option<T>, EnvOptionError<T::Err>> where T: FromStr + Debug, T::Err: Error {
    get::<T,T>(var_name, OptionType::Optional)
}

#[cfg(test)]
mod tests {
    pub use super::*;
    use std::env;

    pub const UNSET_OPTION : &'static str = "__ENVOPTION_TEST_NOT_SET";
    pub const SET_OPTION : &'static str = "__ENVOPTION_TEST_OPTION";

    pub fn set_env() {
        env::set_var(SET_OPTION, "10");
    }

    mod get {
        pub use super::*;

        mod var_not_present {
            use super::*;

            #[test]
            fn required() {
                assert_eq!(get::<String, String>(UNSET_OPTION, OptionType::Required), Result::Err(EnvOptionError::Missing(String::from(UNSET_OPTION))));
            }

            #[test]
            fn optional() {
                assert_eq!(get::<String,String>(UNSET_OPTION, OptionType::Optional), Result::Ok(None));
            }

            #[test]
            fn default_value() {
                assert_eq!(get(UNSET_OPTION, OptionType::Default(String::from("abcdef"))), Result::Ok(Some(String::from("abcdef"))));
            }

            #[test]
            fn default_value_str() {
                assert_eq!(get(UNSET_OPTION, OptionType::Default("abcdef")), Result::Ok(Some(String::from("abcdef"))));
            }

             #[test]
            fn required_usize() {
                assert_eq!(get::<usize, usize>(UNSET_OPTION, OptionType::Required), Result::Err(EnvOptionError::Missing(String::from(UNSET_OPTION))));
            }

            #[test]
            fn optional_usize() {
                assert_eq!(get::<usize, usize>(UNSET_OPTION, OptionType::Optional), Result::Ok(None));
            }

            #[test]
            fn default_value_usize() {
                assert_eq!(get(UNSET_OPTION, OptionType::Default(50)), Result::Ok(Some(50)));
            }
        }

        mod var_is_present {
            use super::*;

            #[test]
            fn required() {
                set_env();
                assert_eq!(get::<String, String>(SET_OPTION, OptionType::Required), Result::Ok(Some(String::from("10"))));
            }

            #[test]
            fn optional() {
                set_env();
                assert_eq!(get::<String, String>(SET_OPTION, OptionType::Optional), Result::Ok(Some(String::from("10"))));
            }

            #[test]
            fn default_value() {
                set_env();
                assert_eq!(get(SET_OPTION, OptionType::Default(String::from("abcdef"))), Result::Ok(Some(String::from("10"))));
            }

            #[test]
            fn default_value_str() {
                set_env();
                assert_eq!(get(SET_OPTION, OptionType::Default("abcdef")), Result::Ok(Some(String::from("10"))));
            }

            #[test]
            fn required_usize() {
                set_env();
                assert_eq!(get::<usize, usize>(SET_OPTION, OptionType::Required), Result::Ok(Some(10)));
            }

            #[test]
            fn optional_usize() {
                set_env();
                assert_eq!(get::<usize, usize>(SET_OPTION, OptionType::Optional), Result::Ok(Some(10)));
            }

            #[test]
            fn default_value_usize() {
                set_env();
                assert_eq!(get(SET_OPTION, OptionType::Default(25)), Result::Ok(Some(10)));
            }
        }
    }

    mod with_default {
       pub use super::*;

        mod var_not_present {
            use super::*;

            #[test]
            fn default_string() {
                assert_eq!(with_default(UNSET_OPTION, String::from("abcdef")), Result::Ok(String::from("abcdef")));
            }

            #[test]
            fn default_str() {
                assert_eq!(with_default(UNSET_OPTION, "abcdef"), Result::Ok(String::from("abcdef")));
            }

            #[test]
            fn with_default_usize() {
                assert_eq!(with_default(UNSET_OPTION, 20), Result::Ok(20));
            }
        }

        mod var_is_present {
            use super::*;

            #[test]
            fn default_string() {
                set_env();
                assert_eq!(with_default(SET_OPTION, String::from("abcdef")), Result::Ok(String::from("10")));
            }

            #[test]
            fn default_str() {
                set_env();
                assert_eq!(with_default(SET_OPTION, "abcdef"), Result::Ok(String::from("10")));
            }

            #[test]
            fn default_usize() {
                set_env();
                assert_eq!(with_default(SET_OPTION, 20), Result::Ok(10));
            }
        }
    }

    mod optional {
        pub use super::*;

        #[test]
        fn var_not_present() {
            assert_eq!(optional::<String>(UNSET_OPTION), Result::Ok(None));
        }

        #[test]
        fn var_is_present() {
            set_env();
            assert_eq!(optional::<String>(SET_OPTION), Result::Ok(Some(String::from("10"))));
        }
    }

    mod require {
        pub use super::*;

        #[test]
        fn var_not_present() {
            assert_eq!(require::<String>(UNSET_OPTION), Result::Err(EnvOptionError::Missing(String::from(UNSET_OPTION))));
        }

        #[test]
        fn var_is_present() {
            set_env();
            assert_eq!(require::<String>(SET_OPTION), Result::Ok(String::from("10")));
        }
    }

}
