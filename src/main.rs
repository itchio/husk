use com::{
    com_interface,
    interfaces::iunknown::IUnknown,
    runtime::{create_instance, init_runtime},
    sys::{HRESULT, IID, S_OK},
};
use widestring::U16CString;

use std::{ffi::c_void, os::raw::c_int, ptr};

type DWORD = u32;
const SLGP_RAWPATH: DWORD = 0x4;
const STGM_READ: DWORD = 0x0;

#[com_interface("000214F9-0000-0000-C000-000000000046")]
pub trait IShellLinkW: IUnknown {
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
    unsafe fn set_description(&self) -> !;

    unsafe fn get_working_directory(&self) -> !;
    unsafe fn set_working_directory(&self) -> !;

    unsafe fn get_arguments(&self) -> !;
    unsafe fn set_arguments(&self) -> !;

    unsafe fn get_hotkey(&self) -> !;
    unsafe fn set_hotkey(&self) -> !;

    unsafe fn get_show_cmd(&self) -> !;
    unsafe fn set_show_cmd(&self) -> !;

    unsafe fn get_icon_location(&self) -> !;
    unsafe fn set_icon_location(&self) -> !;

    unsafe fn get_relative_path(&self) -> !;
    unsafe fn set_relative_path(&self) -> !;

    unsafe fn resolve(&self) -> !;
    unsafe fn set_path(&self) -> !;
}

#[com_interface("0000010C-0000-0000-C000-000000000046")]
pub trait IPersist: IUnknown {
    unsafe fn get_class_id(&self, pclass_id: *mut IID) -> HRESULT;
}

#[com_interface("0000010B-0000-0000-C000-000000000046")]
pub trait IPersistFile: IPersist {
    unsafe fn is_dirty(&self) -> !;
    unsafe fn load(&self, psz_file_name: *const u16, mode: DWORD) -> HRESULT;
    unsafe fn save(&self) -> !;
    unsafe fn save_completed(&self) -> !;
    unsafe fn get_cur_file(&self) -> !;
}

const CLSID_SHELL_LINK: IID = IID {
    data1: 0x00021401,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

fn main() {
    println!("Press enter to continue...");
    use std::io::BufRead;
    let mut s = String::new();
    std::io::stdin().lock().read_line(&mut s).unwrap();

    init_runtime().unwrap();
    println!("Initialized runtime");

    let i = create_instance::<dyn IShellLinkW>(&CLSID_SHELL_LINK).unwrap();
    println!("Got a ShellLink!");

    let pf = i.get_interface::<dyn IPersistFile>().unwrap();
    println!("Got as a PersistFile!");

    let path = r#"C:\Users\amos\Desktop\WizTree.lnk"#;
    let meta = std::fs::metadata(&path).unwrap();
    println!("meta = {:?}", meta);

    let u16s = U16CString::from_str(path).unwrap();
    unsafe {
        let res = pf.load(u16s.as_ptr(), STGM_READ);
        assert_eq!(res, S_OK);
    }
    println!("called load!");

    let mut v = vec![0u16; 256];
    unsafe {
        let res = i.get_path(
            v.as_mut_ptr(),
            v.len() as c_int,
            ptr::null_mut(),
            SLGP_RAWPATH,
        );
        println!("called get_path, checking now...");
        // println!("v = {:?}", v);
        println!("res = {:?}", res);
    }
    let s = U16CString::from(u16s.clone());
    println!("s = {:?}", s.to_string_lossy());

    unsafe {
        let res = i.get_description(v.as_mut_ptr(), v.len() as c_int);
        println!("called get_description, checking now...");
        // println!("v = {:?}", v);
        println!("res = {:?}", res);
    }
    let s = U16CString::from(u16s.clone());
    println!("s = {:?}", s.to_string_lossy());
}
