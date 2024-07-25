use std::ffi::{c_char, CString};

extern "C" {
    fn api_camera_init() -> i8;
    fn api_camera_start() -> i8;
    fn api_camera_stop() -> i8;
    fn api_camera_good_images() -> i32;
    fn api_camera_bad_images() -> i32;
    fn api_camera_set_exposure(val: i32) -> i32;
    fn api_camera_save_file(s: *const c_char) -> i32;
}

pub fn init_camera() -> bool {
    unsafe { api_camera_init() != 0 }
}

pub fn start_camera() -> bool {
    unsafe { api_camera_start() != 0 }
}

pub fn stop_camera() -> bool {
    unsafe { api_camera_stop() != 0 }
}

pub fn get_image_counts() -> (i32, i32) {
    unsafe { (api_camera_good_images(), api_camera_bad_images()) }
}

pub fn save_image(path: &str) -> bool {
    unsafe {
        if let Ok(c_path) = CString::new(path) {
            api_camera_save_file(c_path.as_ptr()) == 1
        } else {
            false
        }
    }
}

pub fn set_exposure(milliseconds: i32) -> bool {
    unsafe { api_camera_set_exposure(milliseconds) != 0 }
}
