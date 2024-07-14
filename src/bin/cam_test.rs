use std::{thread, time::Duration};

use leed_controller::camera::*;

fn main() {
    if !init_camera() {
        println!("Camera init failed!");
        return;
    }

    if !start_camera() {
        println!("Camera start failed!");
        return;
    }

    println!("Camera initialized!");
    loop {
        let (good, bad) = get_image_counts();
        println!("Images: good({}), bad({})", good, bad);
        thread::sleep(Duration::from_secs(10));
        if save_image("test_image.jpg") {
            println!("Saved image!");
        }
    }
}
