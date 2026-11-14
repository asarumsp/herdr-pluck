//! Per-user private directory for picker launch files.
//!
//! Snapshots hold visible pane text, so they must not land world-readable in a shared `/tmp`
//! under predictable names. Everything goes under `$XDG_RUNTIME_DIR/herdr-pluck-<uid>` (or the
//! system temp dir when that is unset), created `0700` and verified to be a real directory owned
//! by the current user. Other users can neither read the files nor plant symlinks in it.

use std::fs::{self, DirBuilder};
use std::io;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::PathBuf;

/// Returns the private directory, creating it on first use.
pub(crate) fn private_dir() -> io::Result<PathBuf> {
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(std::env::temp_dir);
    // SAFETY: getuid has no preconditions and cannot fail.
    let uid = unsafe { libc::getuid() };
    let dir = base.join(format!("herdr-pluck-{uid}"));

    match DirBuilder::new().mode(0o700).create(&dir) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }

    // symlink_metadata: a symlink planted at this path must be rejected, not followed.
    let meta = fs::symlink_metadata(&dir)?;
    if !meta.is_dir() || meta.uid() != uid || meta.mode() & 0o077 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "refusing insecure herdr-pluck directory {} (must be a 0700 directory owned by uid {uid})",
                dir.display()
            ),
        ));
    }
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_dir_is_owner_only() {
        let dir = private_dir().unwrap();
        let meta = fs::symlink_metadata(&dir).unwrap();

        assert!(meta.is_dir());
        assert_eq!(meta.mode() & 0o777, 0o700);
    }
}
