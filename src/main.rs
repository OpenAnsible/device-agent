
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
    println!("OS: {}", os );
    
    let screens = devices::screen::display::list();
    println!("{:?}", screens );
    // for screen in screens.iter() {
    //     println!("Screen ID: {}", screen.id);
    //     println!("Screen Windows:");
    //     for window in screen.windows.iter() {
    //         println!("\t{:?}", window);
    //     }
    // }
}




// fn main() {
//     let s = get_screenshot(0).unwrap();

//     println!("{} x {} x {} = {} bytes", s.height(), s.width(), s.pixel_width(), s.raw_len());

//     let origin = s.get_pixel(0, 0);
//     println!("(0,0): R: {}, G: {}, B: {}", origin.r, origin.g, origin.b);

//     let end_col = s.get_pixel(0, s.width()-1);
//     println!("(0,end): R: {}, G: {}, B: {}", end_col.r, end_col.g, end_col.b);

//     let opp = s.get_pixel(s.height()-1, s.width()-1);
//     println!("(end,end): R: {}, G: {}, B: {}", opp.r, opp.g, opp.b);

//     // WARNING rust-bmp params are (width, height)
//     let mut img = Image::new(s.width() as u32, s.height() as u32);
//     for row in (0..s.height()) {
//         for col in (0..s.width()) {
//             let p = s.get_pixel(row, col);
//             // WARNING rust-bmp params are (x, y)
//             img.set_pixel(col as u32, row as u32, Pixel {r: p.r, g: p.g, b: p.b});
//         }
//     }
//     img.save("test.bmp").unwrap();

//     image::save_buffer("test.png",
//         s.as_ref(), s.width() as u32, s.height() as u32, image::RGBA(8))
//     .unwrap();
// }