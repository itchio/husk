mod interfaces;
use interfaces::*;

use com::{
    runtime::{create_instance, init_runtime},
    sys::{FAILED, HRESULT},
};
use widestring::U16CString;

// unfortunately, paths/descriptions/etc. of ShellLinks are all
// constrained to `MAX_PATH`.
const MAX_PATH: usize = 260;

struct SimpleError(String);

impl SimpleError {
    pub unsafe fn ret(self, p: *mut *mut XString) {
        let xs = XString { data: self.0 };
        *p = Box::into_raw(Box::new(xs));
    }
}

impl<T> From<T> for SimpleError
where
    T: std::error::Error,
{
    fn from(e: T) -> Self {
        Self(format!("{}", e))
    }
}

trait FromUtf16 {
    fn from_utf16(self, method: &str) -> Result<String, SimpleError>;
}

impl FromUtf16 for Vec<u16> {
    fn from_utf16(self, method: &str) -> Result<String, SimpleError> {
        let s = U16CString::from_vec_with_nul(self)
            .map_err(|_| SimpleError(format!("missing null terminator in {} result", method)))?;
        Ok(s.to_string()
            .map_err(|_| SimpleError(format!("invalid utf-16 data in {} result", method)))?)
    }
}

trait Checked
where
    Self: Sized + Copy,
{
    fn check(self, method: &str) -> Result<(), SimpleError>;
    fn format(self, method: &str) -> SimpleError;
}

impl Checked for HRESULT {
    fn check(self, method: &str) -> Result<(), SimpleError> {
        if FAILED(self) {
            Err(self.format(method))
        } else {
            Ok(())
        }
    }

    fn format(self, method: &str) -> SimpleError {
        SimpleError(format!(
            "{} returned system error {} (0x{:x})",
            method, self, self
        ))
    }
}

#[repr(i32)]
pub enum ReturnCode {
    Ok = 0,
    Error = 1,
}

/// Handle for an IShellLink instance
pub struct ShellLink {
    instance: com::ComRc<dyn IShellLinkW>,
}

/// Exchange string: passes Rust strings to C
pub struct XString {
    data: String,
}

impl From<String> for XString {
    fn from(data: String) -> Self {
        Self { data }
    }
}

impl ShellLink {
    fn new() -> Result<Self, SimpleError> {
        init_runtime().map_err(|hr| hr.check("init_runtime").unwrap_err())?;
        let instance = create_instance::<dyn IShellLinkW>(&CLSID_SHELL_LINK)
            .map_err(|hr| hr.check("create_instance<IShellLinkW>").unwrap_err())?;
        Ok(Self { instance })
    }

    fn load(&self, path: &str) -> Result<(), SimpleError> {
        let pf = self.instance.get_interface::<dyn IPersistFile>().unwrap();
        let path = U16CString::from_str(path)?;
        unsafe { pf.load(path.as_ptr(), STGM_READ) }.check("IPersistFile::Load")?;
        Ok(())
    }

    fn get_path(&self) -> Result<String, SimpleError> {
        let mut v = vec![0u16; MAX_PATH + 1];
        unsafe {
            self.instance.get_path(
                v.as_mut_ptr(),
                v.len() as _,
                std::ptr::null_mut(),
                SLGP_RAWPATH,
            )
        }
        .check("IShellLinkW::GetPath")?;
        Ok(v.from_utf16("IShellLinkW::GetPath")?)
    }
}

macro_rules! checked {
    ($x: expr, $p_err: ident) => {
        match $x {
            Err(e) => {
                SimpleError::from(e).ret($p_err);
                return ReturnCode::Error;
            }
            Ok(x) => x,
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn shell_link_new(
    p_link: *mut *mut ShellLink,
    p_err: *mut *mut XString,
) -> ReturnCode {
    let sl = checked!(ShellLink::new(), p_err);
    *p_link = Box::into_raw(Box::new(sl));
    ReturnCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn shell_link_load(
    link: *mut ShellLink,
    path_data: *mut u8,
    path_len: usize,
    p_err: *mut *mut XString,
) -> ReturnCode {
    let v = std::slice::from_raw_parts(path_data, path_len);
    let path = checked!(std::str::from_utf8(v), p_err);
    checked!((*link).load(path), p_err);
    ReturnCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn shell_link_get_path(
    link: *mut ShellLink,
    path: *mut *mut XString,
    p_err: *mut *mut XString,
) -> ReturnCode {
    let res = checked!((*link).get_path(), p_err);
    *path = Box::into_raw(Box::new(XString::from(res)));
    ReturnCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn shell_link_free(link: *mut ShellLink) {
    drop(Box::from_raw(link))
}

#[no_mangle]
pub unsafe extern "C" fn xstring_free(xs: *mut XString) {
    drop(Box::from_raw(xs))
}

#[no_mangle]
pub unsafe extern "C" fn xstring_data(xs: *mut XString) -> *const u8 {
    (*xs).data.as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn xstring_len(xs: *mut XString) -> usize {
    (*xs).data.len()
}
