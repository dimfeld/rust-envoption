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

#[derive(Debug)]
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
    #[test]
    fn it_works() {
    }
}
