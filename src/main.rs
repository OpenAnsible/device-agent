#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, unused_must_use, non_snake_case)]

// extern crate ffmpeg;
#[allow(unused_imports)]
extern crate libc;

use std::thread;
use std::env;
use std::time::{ Duration, SystemTime };
use std::fs::{ File, OpenOptions };
use std::io::Write;
use std::sync::Arc;

pub mod devices;
use devices::screen::CGImage;

#[allow(unused_imports)]
#[link(name="avtools", kind="static")]
extern {
    fn hello();
}


fn window_list (){
    let screens = devices::screen::screens();
    for screen in screens.iter() {
        println!("Screen: {}X{} {:?}", screen.width(), screen.height(), screen);
        // Windows of Display
        let windows = screen.windows();
        match screen.capture(){
            Ok(image) => println!("Screen Width: {:?} Height: {:?}", image.width(), image.height() ),
            Err(_)    => print!("")
        };
        
        for window in windows.iter() {
            println!("\t{:?} {}X{} \t{}\t{}", window.owner_pid(), window.width(), window.height(), window.name(), window.owner_name());
        }
    }
}

fn guess_os() -> String {
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
    os.to_string()
}

#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, unused_must_use, non_snake_case)]
fn main (){
    unsafe{
        // Test C FFI.
        hello();
    };
    window_list();
    return ();
    // Displays
    let screens = devices::screen::screens();
    let display_id = screens[0].id();

    let s     = 25*10;
    let fps   = 25;
    let mut t = 0;
    let mut n = 0;
    let display_id = Arc::new(display_id);

    while t < s {
        let now = SystemTime::now();
        let display_id_clone = display_id.clone();

        // thread::spawn(move || {
        // });
        // thread::sleep(Duration::from_millis(1000/fps));
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