
pub mod devices;



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
    for display_id in screens.iter() {
        println!("Screen ID: {}", display_id);
        println!("Screen Windows:");
        // Windows of Display
        let windows = devices::screen::window_list(*display_id);
        for window in windows.iter() {
            println!("\t{:?} {}\t{}", window.owner_pid, window.name, window.owner_name);
        }
    }

    // Capture
    let image = devices::screen::display_capture(screens[0]);
    match image {
        Ok(image) => {
            image.to_png("test.png");
            image.to_bmp("test.bmp");
        },
        Err(_) => {
            println!("Display Capture Fail.");
        }
    }
}