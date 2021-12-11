use rand::{self, Rng, SeedableRng};
use rand::{distributions::Alphanumeric, rngs::SmallRng};
use std::{cell::UnsafeCell, ffi::{OsStr, OsString}, io};
use std::path::{Path, PathBuf};
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