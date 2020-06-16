use com::{
    com_interface,
    interfaces::iunknown::IUnknown,
    sys::{HRESULT, IID},
};

use std::{ffi::c_void, os::raw::c_int};

pub type DWORD = u32;
pub const SLGP_RAWPATH: DWORD = 0x4;
pub const STGM_READ: DWORD = 0x0;

#[com_interface("000214F9-0000-0000-C000-000000000046")]
pub trait IShellLinkW: IUnknown {
    // DO NOT REORDER THESE - they must match the ABI
    unsafe fn get_path(
        &self,
        psz_file: *mut u16,
        cch: c_int,
        pfd: *const c_void,
        fflags: DWORD,
    ) -> HRESULT;

    unsafe fn get_id_list(&self) -> !;
    unsafe fn set_id_list(&self) -> !;

    unsafe fn get_description(&self, psz_name: *mut u16, cch: c_int) -> HRESULT;
    unsafe fn set_description(&self, psz_name: *const u16) -> HRESULT;

    unsafe fn get_working_directory(&self, psz_dir: *mut u16, cch: c_int) -> HRESULT;
    unsafe fn set_working_directory(&self, psz_dir: *const u16) -> HRESULT;

    unsafe fn get_arguments(&self, psz_args: *mut u16, cch: c_int) -> HRESULT;
    unsafe fn set_arguments(&self, psz_args: *const u16) -> HRESULT;

    unsafe fn get_hotkey(&self) -> !;
    unsafe fn set_hotkey(&self) -> !;

    unsafe fn get_show_cmd(&self) -> !;
    unsafe fn set_show_cmd(&self) -> !;

    unsafe fn get_icon_location(
        &self,
        psz_icon_path: *mut u16,
        cch: c_int,
        pi_icon: *mut c_int,
    ) -> HRESULT;
    unsafe fn set_icon_location(&self, psz_icon_path: *const u16, i_icon: c_int) -> HRESULT;

    unsafe fn set_relative_path(&self) -> !;

    unsafe fn resolve(&self) -> !;
    unsafe fn set_path(&self, psz_file: *const u16) -> HRESULT;
}

#[com_interface("0000010C-0000-0000-C000-000000000046")]
pub trait IPersist: IUnknown {
    // DO NOT REORDER THESE - they must match the ABI
    unsafe fn get_class_id(&self, pclass_id: *mut IID) -> HRESULT;
}

#[com_interface("0000010B-0000-0000-C000-000000000046")]
pub trait IPersistFile: IPersist {
    // DO NOT REORDER THESE - they must match the ABI
    unsafe fn is_dirty(&self) -> !;
    unsafe fn load(&self, psz_file_name: *const u16, mode: DWORD) -> HRESULT;
    unsafe fn save(&self, psz_file_name: *const u16, fremember: bool) -> HRESULT;
    unsafe fn save_completed(&self) -> !;
    unsafe fn get_cur_file(&self) -> !;
}

pub const CLSID_SHELL_LINK: IID = IID {
    data1: 0x00021401,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};
