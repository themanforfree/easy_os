//! Load ELF binaries into memory

use alloc::vec::Vec;
use lazy_static::lazy_static;
use log::info;

lazy_static! {
    /// A global instance of the process loader.
    pub static ref PROC_LOADER: ProcLoader = ProcLoader::new();
}

pub struct ProcLoader {
    app_pos: &'static [usize],
    app_names: Vec<&'static str>,
}

impl ProcLoader {
    pub fn new() -> Self {
        unsafe extern "C" {
            fn _num_apps();
            fn _app_names();
        }
        let num_apps_addr = _num_apps as usize;
        let num_apps_ptr = num_apps_addr as *const usize;
        let num_apps = unsafe { num_apps_ptr.read_volatile() };
        let app_pos = unsafe { core::slice::from_raw_parts(num_apps_ptr.add(1), num_apps + 1) };

        let mut start_ptr = _app_names as usize as *const u8;
        let mut app_names = Vec::new();
        for _ in 0..num_apps {
            let mut end_ptr = start_ptr;
            unsafe {
                while end_ptr.read_volatile() != 0 {
                    end_ptr = end_ptr.add(1);
                }
                let slice =
                    core::slice::from_raw_parts(start_ptr, end_ptr as usize - start_ptr as usize);
                let str = core::str::from_utf8(slice).unwrap();
                app_names.push(str);
                start_ptr = end_ptr.add(1);
            }
        }
        Self { app_pos, app_names }
    }

    pub fn get_app_data(&self, app_id: usize) -> Option<&'static [u8]> {
        if app_id >= self.app_names.len() {
            return None;
        }
        let ptr = self.app_pos[app_id] as *const u8;
        let len = self.app_pos[app_id + 1] - self.app_pos[app_id];
        unsafe { Some(core::slice::from_raw_parts(ptr, len)) }
    }

    pub fn get_app_data_by_name(&self, app_name: &str) -> Option<&'static [u8]> {
        (0..self.app_names.len())
            .find(|&i| self.app_names[i] == app_name)
            .and_then(|i| self.get_app_data(i))
    }

    pub fn list_apps(&self) {
        info!("/**** APPS ****");
        for app in self.app_names.iter() {
            info!("{app}");
        }
        info!("**************/");
    }
}
