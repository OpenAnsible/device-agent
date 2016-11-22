#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, unused_must_use)]
#[allow(non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, non_snake_case)]

#[allow(unused_imports)]
extern crate libc;
#[allow(unused_imports)]
extern crate ffmpeg_sys;

#[allow(unused_imports)]
use ffmpeg_sys::{
    av_malloc, 

    AVPixelFormat, AV_PIX_FMT_YUV420P, AV_PIX_FMT_RGB24, AV_PIX_FMT_BGRA,
    
    av_image_alloc, av_image_fill_arrays,

    AVFrame, AVFrameSideData, AVFrameSideDataType, 
    av_frame_alloc, av_frame_free, av_frame_ref, av_read_frame,
    av_frame_clone, av_frame_unref, av_frame_move_ref, 
    av_frame_get_buffer, av_frame_copy, av_frame_get_plane_buffer,

    SwsContext, SwsFilter, SwsVector, SWS_BICUBIC, SWS_BICUBLIN, SWS_FAST_BILINEAR, SWS_POINT,
    sws_alloc_context, sws_init_context, sws_freeContext, sws_getContext, sws_scale,

    AVFormatContext, avformat_alloc_context, avformat_open_input, 

    avcodec_find_decoder, avcodec_open2, avcodec_decode_video2
};

#[allow(unused_imports)]
use std::{ thread, env, ptr, slice };
#[allow(unused_imports)]
use std::time::{ Duration, SystemTime };
#[allow(unused_imports)]
use std::fs::{ File, OpenOptions };
#[allow(unused_imports)]
use std::io::Write;
#[allow(unused_imports)]
use std::sync::Arc;

pub mod devices;
#[allow(unused_imports)]
use devices::screen::CGImage;

#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, unused_must_use)]
#[allow(non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, non_snake_case)]
fn window_list (){
    let screens = devices::screen::screens();
    for screen in screens.iter() {
        println!("Screen: {}X{} {:?}", screen.width(), screen.height(), screen);
        // Windows of Display
        let windows = screen.windows();
        match screen.capture(){
            Ok(image) => println!("ScreenShot Width: {:?} Height: {:?}", image.width(), image.height() ),
            Err(_)    => print!("")
        };
        
        for window in windows.iter() {
            println!("\t{:?} {}x{} \t{}\t{}\t{:?}", window.owner_pid(), window.width(), window.height(), 
                window.name(), window.owner_name(), window.alpha().unwrap() );
        }
    }
}

#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, unused_must_use)]
#[allow(non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, non_snake_case)]
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

#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, unused_must_use)]
#[allow(non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, non_snake_case)]
#[link(name="avtools")]
extern "C" {
    fn hello();
    // fn rgb24_to_yuv420p(rgb24_data: *const u8, src_w: libc::c_int, src_h: libc::c_int,
    //                     dst_data: *mut *const u8, dst_w: libc::c_int, dst_h: libc::c_int);
    // fn rgb24_to_yuv420p_t(rgb24_data: *const u8, src_w: i32, 
    //                      src_h: i32, dst_data: *mut *const u8);
}

#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, unused_must_use)]
#[allow(non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, non_snake_case)]
fn capture (){
    let screens = devices::screen::screens();
    let image   = screens[0].capture().unwrap();

    // let raw_len = image.raw_len();
    // let pixel_width = image.pixel_width();

    // println!("Pixel Width: {:?}", pixel_width );
    let width  = image.width();
    let height = image.height();

    unsafe{
        let pixels  = image.raw_mut_data(); // BGRA
        let mut src_frame = av_frame_alloc().as_mut().unwrap();
        let mut dst_frame = av_frame_alloc().as_mut().unwrap();
        src_frame.width   = width as i32;
        src_frame.height  = height as i32;
        
        dst_frame.width  = src_frame.width;
        dst_frame.height = src_frame.height;

        let src_pix_fmt  = ffmpeg_sys::AV_PIX_FMT_BGRA;
        let dst_pix_fmt  = ffmpeg_sys::AV_PIX_FMT_YUV420P;

        av_image_alloc(src_frame.data.as_mut_ptr(), src_frame.linesize.as_mut_ptr(),
                       src_frame.width, src_frame.height, 
                       src_pix_fmt, 1 );

        src_frame.data[0] = pixels;

        av_image_alloc(dst_frame.data.as_mut_ptr(), dst_frame.linesize.as_mut_ptr(),
                       dst_frame.width, dst_frame.height,
                       dst_pix_fmt, 1);

        let sws_ctx = sws_getContext(src_frame.width, src_frame.height, src_pix_fmt, 
                        dst_frame.width, dst_frame.height, dst_pix_fmt, 
                        SWS_POINT, ptr::null_mut() , ptr::null_mut(), ptr::null_mut());

        sws_scale(sws_ctx, src_frame.data.as_ptr() as *const *const u8, src_frame.linesize.as_mut_ptr(),
                 0, src_frame.height, dst_frame.data.as_mut_ptr(), 
                 dst_frame.linesize.as_mut_ptr() );

        sws_freeContext(sws_ctx);
        
        let y = slice::from_raw_parts(dst_frame.data[0], (dst_frame.width*dst_frame.height) as usize);
        let u = slice::from_raw_parts(dst_frame.data[1], (dst_frame.width*dst_frame.height/4) as usize);
        let v = slice::from_raw_parts(dst_frame.data[2], (dst_frame.width*dst_frame.height/4) as usize);
        let mut file = OpenOptions::new().create(true).read(true).append(true).open("me.yuv").unwrap();
        file.write(y);
        file.write(u);
        file.write(v);

        println!("Done: ffplay -f rawvideo -pixel_format yuv420p -video_size {:?}x{:?} me.yuv",
            dst_frame.width, dst_frame.height );
    }
}

#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, unused_must_use)]
#[allow(non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, non_snake_case)]
fn main (){
    window_list();
    return ();

    let now = SystemTime::now();
    match now.elapsed() {
       Ok(elapsed) => {
           println!("elapsed: {}s {}ms ", elapsed.as_secs(), elapsed.subsec_nanos()/1000/1000);
       }
       Err(e) => {
           println!("Error: {:?}", e);
       }
    };

    let s     = 25*60;
    let fps   = 25;
    let mut t = 0;
    let mut n = 0;

    while t < s {
        let now = SystemTime::now();

        capture();
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
}