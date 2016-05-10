
#[cfg(target_os = "macos")]
pub mod api {
    extern crate core_foundation;
    extern crate core_graphics;
    extern crate cocoa;

    extern crate image;

    // https://github.com/servo/core-graphics-rs/blob/master/src/display.rs
    // https://developer.apple.com/library/mac/documentation/GraphicsImaging/Reference/Quartz_Services_Ref/#//apple_ref/c/func/CGDisplayCapture

    use self::core_graphics::display::libc;

    use self::core_graphics::display::{
        CGError, boolean_t, 
        CGDirectDisplayID, CGGetActiveDisplayList, CGMainDisplayID, CGDisplayIsMain, 
        CGDisplayScreenSize, CGDisplayRotation, 
        CGWindowListCopyWindowInfo, kCGWindowListExcludeDesktopElements, kCGNullWindowID, CGWindowID,
        CFDictionaryRef, CFDictionary, CFArrayRef,
    };
    use self::core_graphics::geometry::{ CGSize, CGPoint, CGRect };


    type CGDisplayCount = libc::uint32_t;

    const CGErrorSuccess: CGError = 0;
    const CGErrorFailure: CGError = 1000;
    // Window 



    fn GetWindowList (window_id: usize){

        let windowList = CGWindowListCopyWindowInfo(
            kCGWindowListExcludeDesktopElements, 
            // window_id as CGWindowID
            kCGNullWindowID
        );
        println!("{:?}", windowList);
    }



    // Screen
    fn GetMainDisplayID () -> usize {
        CGMainDisplayID() as usize
    }

    fn GetDisplayScreenSize (display_id: usize) -> CGSize {
        CGDisplayScreenSize(display_id as CGDirectDisplayID)
    }
    fn GetDisplayRotation (display_id: usize) -> f64 {
        CGDisplayRotation(display_id as CGDirectDisplayID) as f64
    }
    fn DisplayIsMain (display_id: usize) -> bool {
        CGDisplayIsMain(display_id as CGDirectDisplayID) as bool
    }
    // fn DisplayCreateImage (display_id: usize) -> () {
    //     let display_image = CGDisplayCreateImage(display_id);
    //     // Get info about image
    //     let width = CGImageGetWidth(display_image) as usize;
    //     let height = CGImageGetHeight(display_image) as usize;
    //     let row_len = CGImageGetBytesPerRow(display_image) as usize;
    //     let pixel_bits = CGImageGetBitsPerPixel(display_image) as usize;
    //     if pixel_bits % 8 != 0 {
    //         return Err("Pixels aren't integral bytes.");
    //     }

    //     // Copy image into a Vec buffer
    //     let cf_data = CGDataProviderCopyData(CGImageGetDataProvider(display_image));
    //     let raw_len = CFDataGetLength(cf_data) as usize;

    //     let res = if width*height*pixel_bits != raw_len*8 {
    //         Err("Image size is inconsistent with W*H*D.")
    //     } else {
    //         let data = slice::from_raw_parts(CFDataGetBytePtr(cf_data), raw_len).to_vec().as_slice();
    //         // Pixels are stored as [ARGB](https://en.wikipedia.org/wiki/ARGB).
    //         // { data: data, height: height, width: width, row_len: row_len, pixel_width: pixel_width/8 }

    //         // let img = image::ImageBuffer::new(width, height);
    //         // for row in 0..height {
    //         //  for col in 0..width {
    //         //      // let p = s.get_pixel(row, col);
    //         //      img.put_pixel(col, row, )
    //         //      img.set_pixel(col as u32, row as u32, Pixel {r: p.r, g: p.g, b: p.b});
    //         //  }
    //         // }
    //         println!("{:?}",  data);
    //         data
    //     };

    //     // Release native objects
    //     CGImageRelease(display_image);
    //     CFRelease(cf_data as *const libc::c_void);

    //     // return res;
    //     ()

    // }

    fn GetActiveDisplayList () -> Vec<usize> {

        let mut count: CGDisplayCount = 0;
        let mut err: CGError;
        err = CGGetActiveDisplayList(0, 0 as *mut CGDirectDisplayID, &mut count );
        if err != CGErrorSuccess {
            return Err("Error getting number of displays.");
        }
        let mut disps: Vec<CGDisplayCount> = Vec::with_capacity(count as usize);
        disps.set_len(count as usize);

        err = CGGetActiveDisplayList(disps.len() as CGDisplayCount, &mut disps[0] as *mut CGDirectDisplayID, &mut count);
        if err != CGErrorSuccess {
            return Err("Error getting list of displays.");
        }
        let screens: Vec<usize> = Vec::new();
        for i in disps{
            screens.push(i as usize);
        }
        screens
    }
}

#[cfg(target_os = "linux")]
pub mod api {

    fn GetWindowList (){

    }
    fn GetMainDisplayID () {

    }
    fn GetActiveDisplayList (){

    }
}

#[cfg(target_os = "windows")]
pub mod api {
    fn GetActiveDisplayList (){

    }
}