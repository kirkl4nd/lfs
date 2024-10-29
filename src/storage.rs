use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use libc::statvfs;
#[cfg(unix)]
use std::ffi::CString;

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use winapi::um::fileapi::GetDiskFreeSpaceExW;
#[cfg(windows)]
use winapi::um::winnt::ULARGE_INTEGER;

pub struct Storage {
    pub path: PathBuf, // PathBuf is more flexible for owned paths
}

impl Storage {
    pub fn new() -> Self {
        // read env variable STORAGE_PATH
        let path = env::var("STORAGE_PATH").expect("STORAGE_PATH must be set");
        // create the directory if it doesn't exist
        fs::create_dir_all(&path).expect("Failed to create storage directory");
        Storage {
            path: PathBuf::from(path),
        }
    }

    pub fn get_used_space(&self) -> Result<u64, Box<dyn Error>> {
        let metadata = fs::metadata(&self.path)?;
        Ok(metadata.len())
    }

    /// Returns the amount of available space in the path directory
    pub fn get_free_space(&self) -> Result<u64, Box<dyn Error>> {
        let path = &self.path;
        let free_space = get_free_space_in_path(path)?;
        Ok(free_space)
    }
}

// HELPER FUNCTIONS TO GET FREE SPACE IN TARGET DIR

#[cfg(unix)]
fn get_free_space_in_path(path: &Path) -> Result<u64, Box<dyn Error>> {
    let c_path = CString::new(path.to_str().ok_or("Invalid path string")?)?;
    let mut stat: statvfs = unsafe { std::mem::zeroed() };

    // Call statvfs system call
    if unsafe { statvfs(c_path.as_ptr(), &mut stat) } == 0 {
        // Available blocks * block size = free space in bytes
        Ok(stat.f_bavail as u64 * stat.f_frsize as u64)
    } else {
        Err("Failed to retrieve filesystem statistics".into())
    }
}

#[cfg(windows)]
fn get_free_space_in_path(path: &Path) -> Result<u64, Box<dyn Error>> {
    let path_wide: Vec<u16> = OsStr::new(path).encode_wide().chain(Some(0)).collect();

    let mut free_bytes_available = ULARGE_INTEGER::default();
    let mut total_bytes = ULARGE_INTEGER::default();
    let mut total_free_bytes = ULARGE_INTEGER::default();

    let result = unsafe {
        GetDiskFreeSpaceExW(
            path_wide.as_ptr(),
            &mut free_bytes_available,
            &mut total_bytes,
            &mut total_free_bytes,
        )
    };

    if result != 0 {
        Ok(free_bytes_available.QuadPart() as u64)
    } else {
        Err("Failed to retrieve disk space information".into())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let storage = Storage {
        path: PathBuf::from("/your/path/here"), // Change to your desired path
    };

    match storage.get_free_space() {
        Ok(free_space) => println!("Free space at path: {} bytes", free_space),
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}
