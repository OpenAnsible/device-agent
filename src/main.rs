extern crate ffmpeg;

use std::thread;
use std::env;
use std::time::{ Duration, SystemTime };
use std::fs::{ File, OpenOptions };
use std::io::Write;
use std::sync::Arc;

pub mod devices;

fn ycbcr_to_rgb(y: f64, cb: f64, cr: f64) -> (u8, u8, u8) {
    let r: f64 = y + 1.402   * (cr-128.0);
    let g: f64 = y - 0.34414 * (cb-128.0) -  0.71414 * (cr-128.0);
    let mut b: f64 = y + 1.772   * (cb-128.0);
    if b < 255.0f64 {
        b += 1.0;
    }
    (r as u8, g as u8, b as u8)
}
fn rgb_to_ycbcr(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let r = r as f64;
    let g = g as f64;
    let b = b as f64;

    let y : f64 = 0.299*r + 0.587*g    + 0.114*b;
    let mut cb: f64 = 128.0   - 0.168736*r - 0.331364*g + 0.5*b;
    let mut cr: f64 = 128.0   + 0.5*r      - 0.418688*g - 0.081312*b;
    if y == 0.0 || y == 1.0 {
        cb = 0.0;
        cr = 0.0;
    }
    // if y > 255.0f64 
    //     || cb > 255.0f64
    //     || cr > 255.0f64 {
    //         println!("[Debug]: Y: {}, Cb: {}, Cr: {}", y, cb, cr );
    // }
    (y as u8, cb as u8, cr as u8)
}

type ImageBuff = (usize, usize, usize, Vec<u8>);

fn bgr_to_yuv (width: &usize, height: &usize, pixel_width: &usize, image_buff: &Vec<u8>) -> Vec<u8> {
    let now = SystemTime::now();
    // let image_buff = data; // BGRA
    // println!("{:?}", pixel_width);
    let width: f64  = *width as f64;
    let height:f64  = *height as f64;

    let size   = (width*height*1.5f64) as usize;
    let mut frame: Vec<u8>  = Vec::with_capacity(size);
    frame.resize(size, 0u8);

    for row in 0usize..height as usize {
        for col in 0usize..width as usize {
            let idx = row * (width as usize * pixel_width) + col * pixel_width;
            let b = image_buff[idx];
            let g = image_buff[idx+1];
            let r = image_buff[idx+2];
            let a = image_buff[idx+3];
            // im.set_pixel( col as u32, row as u32, bmp::Pixel {r: r, g: g, b: b});
            let (y, cb, cr) = rgb_to_ycbcr(r, g, b);
            
            let mut yuv_idx = 0usize;
            if row%2 > 0 {
                yuv_idx = (row*width as usize - width as usize)/4;
            } else {
                yuv_idx = row*(width as usize)/4;
            }

            frame[row*width as usize + col] = y;

            let idx1: usize = (width*height) as usize + yuv_idx;
            frame[idx1 + col/2 ] = cb;

            let idx2 = idx1 + (width*height*0.25f64) as usize;
            frame[idx2 + col/2 ] = cr;
        }
    }
    
    match now.elapsed() {
       Ok(elapsed) => {
           println!("Shot RGB TO YCbCr Elapsed: {}s {}ms ", elapsed.as_secs(), elapsed.subsec_nanos()/1000/1000);
       }
       Err(e) => {
           println!("Error: {:?}", e);
       }
    };
    frame
}


fn write_yuv(frame: &Vec<u8>){
    // let mut file = OpenOptions::new().write(true).append(true).open("test.yuv");
    match OpenOptions::new().write(true).append(true).open("test.yuv") {
        Ok(mut f) => {
            f.write(frame.as_slice());
            f.sync_all();
        },
        Err(..) => {
            println!("write error.");
        }
    };
}


fn window_list (){
    let screens = devices::screen::display_list();
    for display_id in screens.iter() {
        println!("Screen ID: {}", display_id);
        println!("Screen Windows:");
        // Windows of Display
        let windows = devices::screen::window_list(*display_id);
        for window in windows.iter() {
            println!("\t{:?} {}\t{}", window.owner_pid, window.name, window.owner_name);
        }
    }
}
fn main (){
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "bsd") {
        "bsd"
    } else {
        "unknow"
    };
    
    println!("Operator System: {}", os );

    // Displays
    let screens = devices::screen::display_list();

    let display_id = screens[0];

    let s     = 25;
    let fps   = 25;
    let mut t = 0;
    let mut n = 0;
    let display_id = Arc::new(screens[0]);

    while t < s {
        let now = SystemTime::now();
        let display_id_clone = display_id.clone();

        match devices::screen::display_capture(*display_id_clone ){
            Ok((w, h, pb, buff)) => {
                println!("{:?} x {:?} x {:?} = {:?}", w, h, pb, buff.len());
                thread::spawn(move || {
                    let frame = bgr_to_yuv(&w, &h, &pb, &buff);
                    println!("Frame Length: {:?}", frame.len());
                });
            },
            Err(..) => {
                println!("Capture Error.");
            }
        };

        thread::sleep(Duration::from_millis(1000/fps));
        t += 1000/fps;
        n += 1;

        match now.elapsed() {
           Ok(elapsed) => {
               println!("elapsed: {}s {}ms ", elapsed.as_secs(), elapsed.subsec_nanos()/1000/1000);
           }
           Err(e) => {
               println!("Error: {:?}", e);
           }
       }
    }
    println!("N: {:?}", n);
}