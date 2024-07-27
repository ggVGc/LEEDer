use std::fs;

use chrono::{Timelike, Utc};
use log::{error, info};

use crate::{
    camera::{init_camera, start_camera},
    motors_client::{Callbacks, MotorsClient},
};

pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Scanner {
    pub target_pos: Position,
    motors: MotorsClient,
}

impl Scanner {
    pub fn new(motors_port_name: &str) -> Option<Self> {
        if setup_camera() {
            info!("Camera initialized");
        } else {
            error!("Camera init failed!");
        }

        let on_scan_start = || {
            info!("Scan started!");
            if fs::metadata("images").is_ok() {
                info!("Renaming old image dir");
                if fs::rename(
                    "images",
                    format!(
                        "images_{}_{}:{}",
                        Utc::now().date_naive(),
                        Utc::now().time().hour(),
                        Utc::now().time().minute()
                    ),
                )
                .is_err()
                {
                    error!("Failed renaming image dir!");
                }
            }
            match fs::create_dir("images") {
                Ok(()) => (),
                Err(_) => error!("Could not create image directory!"),
            }
        };

        let on_scan_step = |step_size, x: i32, y: i32| {
            // info!("Scan step, {}, {}", x, y);
            let image_dir_path = &format!("images/{:.2}", step_size);
            if fs::metadata(image_dir_path).is_err() {
                match fs::create_dir(image_dir_path) {
                    Ok(()) => (),
                    Err(_) => error!("Could not create image directory for step size!"),
                }
            }

            let path = format!("{}/{}_{}.bmp", image_dir_path, x, y);

            // TODO: There's some conflict with multiple calls to NETUSBCAM_SaveToFile, which is used by
            // both save_image and for live_image.bmp, resulting in images often not being saved.
            // Copy last live image instead for now.

            if fs::copy("live_image.bmp", &path).is_ok() {
                info!("Saved image: {}", path);
            } else {
                error!("Image save failed: {}", path);
            }

            /*
            if save_image(path.as_str()) {
                info!("Saved image: {}", path);
            } else {
                error!("Image save failed: {}", path);
            }
            */
        };

        let motors = MotorsClient::new(
            motors_port_name,
            Callbacks {
                scan_start: on_scan_start,
                scan_step: on_scan_step,
            },
        )
        .ok()?;

        motors.set_conf();

        Some(Self {
            motors,
            target_pos: Position { x: 0, y: 0 },
        })
    }

    pub fn update(&mut self) {
        let old_step_size = self.motors.step_size;

        let on_new_step_size = |step_size: f32| {
            let x = self.target_pos.x as f32 * old_step_size;
            let y = self.target_pos.y as f32 * old_step_size;
            self.target_pos.x = (x / step_size).round() as i32;
            self.target_pos.y = (y / step_size).round() as i32;
        };

        self.motors.update(on_new_step_size);

        let (x_max, y_max) = self.motors.get_limits();
        if self.target_pos.x < 0 {
            self.target_pos.x = 0
        }

        if self.target_pos.y < 0 {
            self.target_pos.y = 0
        }

        if self.target_pos.x >= x_max {
            self.target_pos.x = x_max - 1
        }

        if self.target_pos.y >= y_max {
            self.target_pos.y = y_max - 1
        }
    }

    pub fn start_scan(&self) {
        info!("Requestsing scan start");
        self.motors.start_scan()
    }

    pub fn stop_scan(&self) {
        info!("Requestsing scan stop");
        self.motors.stop_scan()
    }

    pub fn get_scan_pos(&self) -> ((i32, i32), (i32, i32)) {
        (self.motors.get_last_pos(), self.motors.get_limits())
    }

    pub fn goto_target_pos(&self) {
        self.motors.set_pos(self.target_pos.x, self.target_pos.y);
    }

    pub fn adjust_scan_step(&self, amount: f32) {
        self.motors.adjust_step(amount);
    }

    pub fn get_step_size(&self) -> f32 {
        self.motors.step_size
    }
}

pub fn setup_camera() -> bool {
    if !init_camera() {
        error!("Camera init failed!");
        return false;
    }

    if !start_camera() {
        error!("Camera start failed!");
        return false;
    }

    // TODO: Set exposure on start?
    /*
    if !set_exposure(120) {
        println!("Failed setting exposure!");
    }
    */

    true
}
