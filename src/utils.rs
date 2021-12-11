use rand::{self, Rng, SeedableRng};
use rand::{distributions::Alphanumeric, rngs::SmallRng};
use std::{cell::UnsafeCell, ffi::{OsStr, OsString}, io};
use std::path::{Path, PathBuf};
use anyhow::{Result};

thread_local! {
    static THREAD_RNG: UnsafeCell<SmallRng> = UnsafeCell::new(SmallRng::from_entropy());
}

/// Generate a random string for a file name
/// Taken from: https://github.com/Stebalien/tempfile/blob/master/src/util.rs
fn tmpname(prefix: &OsStr, suffix: &OsStr, rand_len: usize) -> OsString {
    let mut buf = OsString::with_capacity(prefix.len() + suffix.len() + rand_len);
    buf.push(prefix);

    // Push each character in one-by-one. Unfortunately, this is the only
    // safe(ish) simple way to do this without allocating a temporary
    // String/Vec.
    THREAD_RNG.with(|rng| unsafe {
        (&mut *rng.get())
            .sample_iter(&Alphanumeric)
            .take(rand_len)
            .for_each(|b| buf.push(std::str::from_utf8_unchecked(&[b as u8])))
    });
    buf.push(suffix);
    buf
}

/// Create a temporary file with a random name
pub fn tmpfile(base: &Path, prefix: &str, suffix: &str) -> io::Result<PathBuf> {
    let mut path = base.to_path_buf();
    path.push(tmpname(OsStr::new(prefix), OsStr::new(suffix), 8));
    Ok(path)
}

/// Returns cwd path relative to the current cwd.
/// If the desired cwd is absolute, it is returned as is.
pub fn resolve_cwd(current: &PathBuf, cwd: Option<&String>) -> PathBuf {
    let cwd = cwd.map(PathBuf::from).unwrap_or_else(|| current.clone());
    if cwd.is_absolute() {
        cwd
    } else {
        current.join(cwd)
    }
}

/// Executes a function with a reserved temporary file
/// The temporary file is deleted when the function returns
pub fn with_tempfile<R>(
    base: &PathBuf,
    f: impl FnOnce(&PathBuf) -> Result<R>,
) -> Result<R> {
    let temp_path = tmpfile(base, "nauman", "")?;

    let result = f(&temp_path);
    if temp_path.exists() {
        if let Err(_e) = std::fs::remove_file(temp_path) {
            // TODO: log error / warning
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use anyhow::anyhow;
    use crate::utils::{resolve_cwd, with_tempfile};

    #[test]
    fn test_with_tempfile() {
        let temp_path = std::env::temp_dir();

        let mut temp_file = PathBuf::new();
        with_tempfile(&temp_path, |file| {
            std::fs::write(file, "test").expect("Failed to write to temp file");
            assert!(file.exists());
            assert!(file.is_file());
            temp_file = file.to_path_buf();
            Ok(())
        }).unwrap();
        assert!(!temp_file.exists());

        let mut temp_file = PathBuf::new();
        let _ = with_tempfile(&temp_path, |file| {
            std::fs::write(file, "test").expect("Failed to write to temp file");
            assert!(file.exists());
            assert!(file.is_file());
            temp_file = file.to_path_buf();
            if false {
                Ok(())
            } else {
                Err(anyhow!("Failed"))
            }
        });
        assert!(!temp_file.exists());
    }

    #[test]
    fn test_resolve_cwd() {
        let base = PathBuf::from("/base");
        let relative = "relative".to_string();
        let absolute = "/absolute".to_string();

        assert_eq!(resolve_cwd(&base,  Some(&relative)), PathBuf::from("/base/relative"));
        assert_eq!(resolve_cwd(&base,  None), PathBuf::from("/base"));
        assert_eq!(resolve_cwd(&base,  Some(&absolute)), PathBuf::from("/absolute"));
    }
}