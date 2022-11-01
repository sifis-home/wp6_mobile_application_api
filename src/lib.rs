use error::Result;
use std::path::PathBuf;
use std::{env, fs};

pub mod configs;
pub mod error;

const SIFIS_HOME_PATH_ENV: &str = "SIFIS_HOME_PATH";

/// Path to SIFIS-home configuration files
///
/// The path is made from the SIFIS_HOME_PATH environment variable when it is available.
/// Otherwise, the function returns the relative path of 'sifis-home'.
///
/// The function creates a directory if it does not exist. If the directory creation fails, the
/// function returns an error.
pub fn sifis_home_path() -> Result<PathBuf> {
    let path = PathBuf::from(match env::var(SIFIS_HOME_PATH_ENV) {
        Ok(path) => path,
        Err(_) => String::from("sifis-home"),
    });
    fs::create_dir_all(&path)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // File I/O is not supported with Miri when isolation is enabled
    fn home_path() {
        // Testing default path
        env::remove_var(SIFIS_HOME_PATH_ENV);
        let path = sifis_home_path().unwrap();
        assert_eq!(path.to_str().unwrap(), "sifis-home");
        let _ = fs::remove_dir(path);

        // Testing with environment variable
        let test_path = "sifis-home-test";
        env::set_var(SIFIS_HOME_PATH_ENV, test_path);
        let path = sifis_home_path().unwrap();
        assert_eq!(path.to_str().unwrap(), test_path);
        let _ = fs::remove_dir(path);
    }
}
