mod interfaces;
mod runtime;
use interfaces::*;

use com::{
    runtime::create_instance,
    sys::{FAILED, HRESULT},
};
use runtime::ComGuard;
use std::path::{Path, PathBuf};
use widestring::U16CString;

// unfortunately, paths/descriptions/etc. of ShellLinks are all
// constrained to `MAX_PATH`.
// TODO: figure out how this works with LongPathAware?
const MAX_PATH: usize = 260;

const INFOTIPSIZE: usize = 1024;

#[derive(Debug)]
pub struct SimpleError(String);

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

struct NulTerminatedUtf16(Vec<u16>);

impl NulTerminatedUtf16 {
    fn to_u16_c_string(self, method: &str) -> Result<U16CString, SimpleError> {
        let res = U16CString::from_vec_with_nul(self.0)
            .map_err(|_| SimpleError(format!("input is not null-terminated in {}", method)))?;
        Ok(res)
    }

    fn to_string(self, method: &str) -> Result<String, SimpleError> {
        let res = self
            .to_u16_c_string(method)?
            .to_string()
            .map_err(|_| SimpleError(format!("invalid utf-16 data in {}", method)))?;
        Ok(res)
    }

    fn to_path_buf(self, method: &str) -> Result<PathBuf, SimpleError> {
        let res = self.to_u16_c_string(method)?;
        let res = res.to_os_string();
        Ok(res.into())
    }
}

pub trait IntoStringSimple {
    fn into_string_simple(self, method: &str) -> Result<String, SimpleError>;
}

impl IntoStringSimple for PathBuf {
    fn into_string_simple(self, method: &str) -> Result<String, SimpleError> {
        let s = self.into_os_string();
        let s = s
            .into_string()
            .map_err(|_| SimpleError(format!("invalid utf-16 data in {} result", method)))?;
        Ok(s)
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
    _guard: ComGuard,
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
    pub fn new() -> Result<Self, SimpleError> {
        let guard = ComGuard::new()?;
        let instance = create_instance::<dyn IShellLinkW>(&CLSID_SHELL_LINK)
            .map_err(|hr| hr.check("create_instance<IShellLinkW>").unwrap_err())?;
        Ok(Self {
            _guard: guard,
            instance,
        })
    }

    pub fn load<P: AsRef<Path>>(&self, path: P) -> Result<(), SimpleError> {
        let pf = self.instance.get_interface::<dyn IPersistFile>().unwrap();
        let path = U16CString::from_os_str(path.as_ref())?;
        unsafe { pf.load(path.as_ptr(), STGM_READ) }.check("IPersistFile::Load")?;
        Ok(())
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), SimpleError> {
        let pf = self.instance.get_interface::<dyn IPersistFile>().unwrap();
        let path = U16CString::from_os_str(path.as_ref())?;
        unsafe { pf.save(path.as_ptr(), false) }.check("IPersistFile::Save")?;
        Ok(())
    }

    pub fn get_path(&self) -> Result<PathBuf, SimpleError> {
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
        Ok(NulTerminatedUtf16(v).to_path_buf("IShellLinkW::GetPath")?)
    }

    /// Sets the shell link's path (what it points to)
    pub fn set_path<P: AsRef<Path>>(&self, path: P) -> Result<(), SimpleError> {
        let path = U16CString::from_os_str(path.as_ref())?;
        unsafe { self.instance.set_path(path.as_ptr()) }.check("IShellLinkW::SetPath")?;
        Ok(())
    }

    pub fn get_arguments(&self) -> Result<String, SimpleError> {
        let mut v = vec![0u16; INFOTIPSIZE + 1];
        unsafe { self.instance.get_arguments(v.as_mut_ptr(), v.len() as _) }
            .check("IShellLinkW::GetArguments")?;
        Ok(NulTerminatedUtf16(v).to_string("IShellLinkW::GetArguments")?)
    }

    pub fn set_arguments(&self, path: &str) -> Result<(), SimpleError> {
        let path = U16CString::from_str(path)?;
        unsafe { self.instance.set_arguments(path.as_ptr()) }.check("IShellLinkW::SetArguments")?;
        Ok(())
    }

    pub fn get_description(&self) -> Result<String, SimpleError> {
        let mut v = vec![0u16; INFOTIPSIZE + 1];
        unsafe { self.instance.get_description(v.as_mut_ptr(), v.len() as _) }
            .check("IShellLinkW::GetDescription")?;
        Ok(NulTerminatedUtf16(v).to_string("IShellLinkW::GetDescription")?)
    }

    pub fn set_description(&self, path: &str) -> Result<(), SimpleError> {
        let path = U16CString::from_str(path)?;
        unsafe { self.instance.set_description(path.as_ptr()) }
            .check("IShellLinkW::SetDescription")?;
        Ok(())
    }

    pub fn get_working_directory(&self) -> Result<PathBuf, SimpleError> {
        let mut v = vec![0u16; INFOTIPSIZE + 1];
        unsafe {
            self.instance
                .get_working_directory(v.as_mut_ptr(), v.len() as _)
        }
        .check("IShellLinkW::GetWorkingDirectory")?;
        Ok(NulTerminatedUtf16(v).to_path_buf("IShellLinkW::GetWorkingDirectory")?)
    }

    pub fn set_working_directory<P: AsRef<Path>>(&self, path: P) -> Result<(), SimpleError> {
        let path = U16CString::from_os_str(path.as_ref().as_os_str())?;
        unsafe { self.instance.set_working_directory(path.as_ptr()) }
            .check("IShellLinkW::SetWorkingDirectory")?;
        Ok(())
    }

    pub fn get_icon_location(&self) -> Result<(PathBuf, i32), SimpleError> {
        let mut v = vec![0u16; INFOTIPSIZE + 1];
        let mut index = 0;
        unsafe {
            self.instance
                .get_icon_location(v.as_mut_ptr(), v.len() as _, &mut index)
        }
        .check("IShellLinkW::GetIconLocation")?;

        let s = NulTerminatedUtf16(v).to_path_buf("IShellLinkW::GetIconLocation")?;
        Ok((s, index as _))
    }

    pub fn set_icon_location<P: AsRef<Path>>(
        &self,
        path: P,
        index: i32,
    ) -> Result<(), SimpleError> {
        let path = U16CString::from_os_str(path.as_ref().as_os_str())?;
        unsafe { self.instance.set_icon_location(path.as_ptr(), index as _) }
            .check("IShellLinkW::SetIconLocation")?;
        Ok(())
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
pub unsafe extern "C" fn shell_link_save(
    link: *mut ShellLink,
    path_data: *mut u8,
    path_len: usize,
    p_err: *mut *mut XString,
) -> ReturnCode {
    let v = std::slice::from_raw_parts(path_data, path_len);
    let path = checked!(std::str::from_utf8(v), p_err);
    checked!((*link).save(path), p_err);
    ReturnCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn shell_link_get_path(
    link: *mut ShellLink,
    path: *mut *mut XString,
    p_err: *mut *mut XString,
) -> ReturnCode {
    let res = checked!((*link).get_path(), p_err);
    let res: String = checked!(res.into_string_simple("shell_link_get_path"), p_err);
    *path = Box::into_raw(Box::new(res.into()));
    ReturnCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn shell_link_set_path(
    link: *mut ShellLink,
    path_data: *mut u8,
    path_len: usize,
    p_err: *mut *mut XString,
) -> ReturnCode {
    let v = std::slice::from_raw_parts(path_data, path_len);
    let path = checked!(std::str::from_utf8(v), p_err);
    checked!((*link).set_path(path), p_err);
    ReturnCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn shell_link_get_arguments(
    link: *mut ShellLink,
    arguments: *mut *mut XString,
    p_err: *mut *mut XString,
) -> ReturnCode {
    let res = checked!((*link).get_arguments(), p_err);
    *arguments = Box::into_raw(Box::new(XString::from(res)));
    ReturnCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn shell_link_set_arguments(
    link: *mut ShellLink,
    arguments_data: *mut u8,
    arguments_len: usize,
    p_err: *mut *mut XString,
) -> ReturnCode {
    let v = std::slice::from_raw_parts(arguments_data, arguments_len);
    let arguments = checked!(std::str::from_utf8(v), p_err);
    checked!((*link).set_arguments(arguments), p_err);
    ReturnCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn shell_link_get_description(
    link: *mut ShellLink,
    description: *mut *mut XString,
    p_err: *mut *mut XString,
) -> ReturnCode {
    let res = checked!((*link).get_description(), p_err);
    *description = Box::into_raw(Box::new(XString::from(res)));
    ReturnCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn shell_link_set_description(
    link: *mut ShellLink,
    description_data: *mut u8,
    description_len: usize,
    p_err: *mut *mut XString,
) -> ReturnCode {
    let v = std::slice::from_raw_parts(description_data, description_len);
    let description = checked!(std::str::from_utf8(v), p_err);
    checked!((*link).set_description(description), p_err);
    ReturnCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn shell_link_get_working_directory(
    link: *mut ShellLink,
    working_directory: *mut *mut XString,
    p_err: *mut *mut XString,
) -> ReturnCode {
    let res = checked!((*link).get_working_directory(), p_err);
    let res: String = checked!(
        res.into_string_simple("shell_link_get_working_directory"),
        p_err
    );
    *working_directory = Box::into_raw(Box::new(res.into()));
    ReturnCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn shell_link_set_working_directory(
    link: *mut ShellLink,
    working_directory_data: *mut u8,
    working_directory_len: usize,
    p_err: *mut *mut XString,
) -> ReturnCode {
    let v = std::slice::from_raw_parts(working_directory_data, working_directory_len);
    let working_directory = checked!(std::str::from_utf8(v), p_err);
    checked!((*link).set_working_directory(working_directory), p_err);
    ReturnCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn shell_link_get_icon_location(
    link: *mut ShellLink,
    icon_location: *mut *mut XString,
    icon_index: *mut i32,
    p_err: *mut *mut XString,
) -> ReturnCode {
    let (res, res_index) = checked!((*link).get_icon_location(), p_err);
    let res = checked!(
        res.into_string_simple("shell_link_get_icon_location"),
        p_err
    );
    *icon_location = Box::into_raw(Box::new(res.into()));
    *icon_index = res_index;
    ReturnCode::Ok
}

#[no_mangle]
pub unsafe extern "C" fn shell_link_set_icon_location(
    link: *mut ShellLink,
    icon_location_data: *mut u8,
    icon_location_len: usize,
    icon_index: i32,
    p_err: *mut *mut XString,
) -> ReturnCode {
    let v = std::slice::from_raw_parts(icon_location_data, icon_location_len);
    let icon_location = checked!(std::str::from_utf8(v), p_err);
    checked!((*link).set_icon_location(icon_location, icon_index), p_err);
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

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::sync::{Mutex, Once};

    // IMPORTANT NOTE: these tests cannot run in parallel in the same
    // process, as they manipulate process-wide COM state.
    static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
    static LOG: Once = Once::new();

    #[test]
    fn test_com_guard() {
        LOG.call_once(pretty_env_logger::init);
        let _lock = TEST_MUTEX.lock().unwrap();

        assert_eq!(false, runtime::is_active());
        let g1 = ComGuard::new().unwrap();
        assert_eq!(true, runtime::is_active());
        let g2 = ComGuard::new().unwrap();
        assert_eq!(true, runtime::is_active());
        drop(g2);
        assert_eq!(true, runtime::is_active());
        drop(g1);
        assert_eq!(false, runtime::is_active());
    }

    #[test]
    fn create_two_shortcuts() {
        LOG.call_once(pretty_env_logger::init);
        let _lock = TEST_MUTEX.lock().unwrap();

        let cwd = std::env::current_dir().unwrap();
        println!("cwd = {:?}", cwd);

        for name in &["Test1.lnk", "Test2.lnk"] {
            let path = cwd.join(name);
            std::fs::remove_file(&path).ok();

            let sl = ShellLink::new().unwrap();
            let target = cwd.join("Cargo.toml");
            sl.set_path(target).unwrap();
            sl.save(&path).unwrap();

            let sl = ShellLink::new().unwrap();
            sl.load(&path).unwrap();

            std::fs::remove_file(&path).unwrap();
        }
    }
}
