use std::path::PathBuf;
use thiserror::Error;
use winreg::RegKey;
use winreg::enums::HKEY_LOCAL_MACHINE;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("cannot find the directory")]
    DirectoryNotFound,
}

pub enum DirectoryType {
    /// Get the path to the binaries.
    Binaries,
    /// Get the path to the headers.
    Headers,
    /// Get the path to the libraries.
    Libraries,
}

pub struct WindowsKits {
    path: PathBuf,
}

impl WindowsKits {
    /// Sets up a new `WindowsKits` instance by querying SOFTWARE\Microsoft\Windows Kits\Installed
    /// Roots for the path to the directory containing the Windows SDKs.
    pub fn new() -> Result<Self, Error> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = r"SOFTWARE\Microsoft\Windows Kits\Installed Roots";
        let dir: String = hklm.open_subkey(key)?.get_value("KitsRoot10")?;

        Ok(Self {
            path: dir.into(),
        })
    }

    /// Returns the path to the Windows Kits directory. The default should be
    /// `C:\Program Files (x86)\Windows Kits\10`.
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    /// Retrieves the path to the directory for the given [`DirectoryType`].
    pub fn get_dir(&self, directory_type: DirectoryType) -> PathBuf {
        self.path()
            .join(match directory_type {
                DirectoryType::Binaries => "bin",
                DirectoryType::Headers => "Include",
                DirectoryType::Libraries => "Lib",
            })
    }

    /// Retrieves the path to the directory for the given [`DirectoryType`] joined by the version
    /// directory, which is selected by enumerating the version directories and picking the highest
    /// version.
    pub fn get_version_dir(&self, directory_type: DirectoryType) -> Result<PathBuf, Error> {
        let dir = self.get_dir(directory_type).read_dir()?;

        let path = dir
            .filter_map(|dir| dir.ok())
            .map(|dir| dir.path())
            .filter(|dir| {
                dir.components()
                    .last()
                    .and_then(|c| c.as_os_str().to_str())
                    .map(|c| c.starts_with("10."))
                    .unwrap_or(false)
            })
            .max()
            .ok_or(Error::DirectoryNotFound)?;

        Ok(path)
    }
}
