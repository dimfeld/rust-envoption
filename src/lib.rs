use std::env;
use std::str::FromStr;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;

/// The behavior if the environment variable is not set.
pub enum OptionType<T> where T: FromStr + Debug, T::Err: Error {
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
pub fn get<T>(var_name: &str, mode: OptionType<T>) -> Result<Option<T>, EnvOptionError<T::Err>>  where T : FromStr + Debug, T::Err: Error {
    match env::var(var_name) {
        Err(_) => match mode {
            OptionType::Optional => Ok(None),
            OptionType::Required => Err(EnvOptionError::Missing(var_name.to_string())),
            OptionType::Default(d) => Ok(Some(d)),
        },
        Ok(value) => value.parse::<T>().map(|value| Some(value)).map_err(|e| EnvOptionError::ParseError(var_name.to_string(), e)),
    }
}

/// Sugar around get to avoid the extra `Option` when it will never be `None` anyway.
pub fn require<T>(var_name: &str, default: Option<T>) -> Result<T, EnvOptionError<T::Err>> where T : FromStr + Debug, T::Err: Error {
    let mode = match default {
        None => OptionType::Required,
        Some(v) => OptionType::Default(v)
    };

    get(var_name, mode).map(|o| o.unwrap())
}

#[cfg(test)]
mod tests {
    pub use super::*;
    mod get {
        pub use super::*;

        mod var_not_present {
            use super::*;

            #[test]
            fn required() {
                assert_eq!(get::<String>("__ENVOPTION_TEST_NOT_SET", OptionType::Required), Result::Err(EnvOptionError::Missing(String::from("__ENVOPTION_TEST_NOT_SET"))));
            }

            #[test]
            fn optional() {
                assert_eq!(get::<String>("__ENVOPTION_TEST_NOT_SET", OptionType::Optional), Result::Ok(None));
            }

            #[test]
            fn default_value() {
                assert_eq!(get("__ENVOPTION_TEST_NOT_SET", OptionType::Default(String::from("abcdef"))), Result::Ok(Some(String::from("abcdef"))));
            }

             #[test]
            fn required_usize() {
                assert_eq!(get::<usize>("__ENVOPTION_TEST_NOT_SET", OptionType::Required), Result::Err(EnvOptionError::Missing(String::from("__ENVOPTION_TEST_NOT_SET"))));
            }

            #[test]
            fn optional_usize() {
                assert_eq!(get::<usize>("__ENVOPTION_TEST_NOT_SET", OptionType::Optional), Result::Ok(None));
            }

            #[test]
            fn default_value_usize() {
                assert_eq!(get("__ENVOPTION_TEST_NOT_SET", OptionType::Default(50)), Result::Ok(Some(50)));
            }
        }

        mod var_is_present {
            use super::*;
            use std::env;

            #[test]
            fn required() {
                env::set_var("__ENV_OPTION_TEST_OPTION", "10");
                assert_eq!(get::<String>("__ENV_OPTION_TEST_OPTION", OptionType::Required), Result::Ok(Some(String::from("10"))));
            }

            #[test]
            fn optional() {
                env::set_var("__ENV_OPTION_TEST_OPTION", "10");
                assert_eq!(get::<String>("__ENV_OPTION_TEST_OPTION", OptionType::Optional), Result::Ok(Some(String::from("10"))));
            }

            #[test]
            fn default_value() {
                env::set_var("__ENV_OPTION_TEST_OPTION", "10");
                assert_eq!(get("__ENV_OPTION_TEST_OPTION", OptionType::Default(String::from("abcdef"))), Result::Ok(Some(String::from("10"))));
            }

            #[test]
            fn required_usize() {
                env::set_var("__ENV_OPTION_TEST_OPTION", "10");
                assert_eq!(get::<usize>("__ENV_OPTION_TEST_OPTION", OptionType::Required), Result::Ok(Some(10)));
            }

            #[test]
            fn optional_usize() {
                env::set_var("__ENV_OPTION_TEST_OPTION", "10");
                assert_eq!(get::<usize>("__ENV_OPTION_TEST_OPTION", OptionType::Optional), Result::Ok(Some(10)));
            }

            #[test]
            fn default_value_usize() {
                env::set_var("__ENV_OPTION_TEST_OPTION", "10");
                assert_eq!(get("__ENV_OPTION_TEST_OPTION", OptionType::Default(25)), Result::Ok(Some(10)));
            }
        }
    }


    mod require {
        pub use super::*;

        mod var_not_present {
            use super::*;

            #[test]
            fn no_default() {
                assert_eq!(require::<String>("__ENVOPTION_TEST_NOT_SET", None), Result::Err(EnvOptionError::Missing(String::from("__ENVOPTION_TEST_NOT_SET"))));
            }

            #[test]
            fn with_default() {
                assert_eq!(require("__ENVOPTION_TEST_NOT_SET", Some(String::from("abcdef"))), Result::Ok(String::from("abcdef")));
            }

            #[test]
            fn no_default_usize() {
                assert_eq!(require::<usize>("__ENVOPTION_TEST_NOT_SET", None), Result::Err(EnvOptionError::Missing(String::from("__ENVOPTION_TEST_NOT_SET"))));
            }

            #[test]
            fn with_default_usize() {
                assert_eq!(require("__ENVOPTION_TEST_NOT_SET", Some(20)), Result::Ok(20));
            }
        }

        mod var_is_present {
            use super::*;
            use std::env;

            #[test]
            fn no_default() {
                env::set_var("__ENV_OPTION_TEST_OPTION", "10");
                assert_eq!(require::<String>("__ENV_OPTION_TEST_OPTION", None), Result::Ok(String::from("10")));
            }

            #[test]
            fn with_default() {
                env::set_var("__ENV_OPTION_TEST_OPTION", "10");
                assert_eq!(require("__ENV_OPTION_TEST_OPTION", Some(String::from("abcdef"))), Result::Ok(String::from("10")));
            }

            #[test]
            fn no_default_usize() {
                env::set_var("__ENV_OPTION_TEST_OPTION", "10");
                assert_eq!(require::<usize>("__ENV_OPTION_TEST_OPTION", None), Result::Ok(10));
            }

            #[test]
            fn with_default_usize() {
                env::set_var("__ENV_OPTION_TEST_OPTION", "10");
                assert_eq!(require("__ENV_OPTION_TEST_OPTION", Some(20)), Result::Ok(10));
            }
        }
    }

}
