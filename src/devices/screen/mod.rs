

pub fn hello () {
    println!("hello, world!");
}

pub fn main (){
    println!("this is main at screen.rs");
}



#[cfg(target_os = "macos")]
pub mod ffi {
    extern crate core_foundation;
    extern crate core_graphics;

    #[derive(Debug)]
    pub struct Window {
        number: isize,
        store_type: isize,
        layer: isize,
        sharing_state: isize,
        alpha: f32,
        owner_pid: isize,
        memory_usage: isize,  // i64
        workspace: isize,
        owner_name: String, // &'static str
        name: String,
        is_on_screen: bool,
        backing_location_video_memory: bool,
        // kCGWindowBounds
        height: f64,
        width: f64,
        x: f64,
        y: f64,
    }
    pub type Windows = Vec<Window>;

    extern crate image;

    // https://github.com/servo/core-graphics-rs/blob/master/src/display.rs
    // https://developer.apple.com/library/mac/documentation/GraphicsImaging/Reference/Quartz_Services_Ref/#//apple_ref/c/func/CGDisplayCapture

    use self::core_graphics::display::libc;

    use self::core_graphics::display::{
        CGError, boolean_t, 
        CGDirectDisplayID, CGGetActiveDisplayList, CGMainDisplayID, CGDisplayIsMain, 
        CGDisplayScreenSize, CGDisplayRotation, 
        CGWindowListCopyWindowInfo, CGWindowID,
        kCGWindowListExcludeDesktopElements, kCGNullWindowID,
        CFDictionaryRef, CFDictionary, CFArrayRef, CFArray, CFIndex, CFTypeRef,
        CFArrayGetCount, // CFArrayGetValueAtIndex,
    };
    use self::core_graphics::geometry::{ CGSize, CGPoint, CGRect, CGFloat };
    use self::core_foundation::dictionary::{ 
        CFDictionaryContainsKey, CFDictionaryGetCount, CFDictionaryGetValueIfPresent, 
        CFDictionaryGetKeysAndValues
    };
    use self::core_foundation::string::{ CFString, CFStringRef, __CFString, CFStringGetCStringPtr, kCFStringEncodingUTF8 };
    use self::core_foundation::number::{ 
        CFNumber, CFNumberRef, __CFNumber, CFNumberType, CFNumberGetValue,
        kCFNumberSInt32Type, kCGWindowIDCFNumberType, kCFNumberSInt64Type, kCFNumberFloat32Type, 
        kCFNumberFloat64Type, kCFNumberIntType, kCFNumberLongType, kCFNumberLongLongType, kCFNumberFloatType,
        CFBooleanGetValue
    };
    use self::core_foundation::boolean:: {
        CFBoolean, CFBooleanRef, kCFBooleanTrue, kCFBooleanFalse
    };
    use std::ffi::{ CString, CStr };
    use std::{ ptr, mem, str };

    type CGDisplayCount = libc::uint32_t;

    const CGErrorSuccess: CGError = 0;
    const CGErrorFailure: CGError = 1000;



    extern {
        pub fn CFArrayGetValueAtIndex(theArray: CFArrayRef, idx: CFIndex) -> CFDictionaryRef;
        pub fn CGRectMakeWithDictionaryRepresentation(dict: CFDictionaryRef, rect: *mut CGRect ) -> bool;
    }
    unsafe fn str_to_cf_dict_key (s: &str) -> &libc::c_void {
        let k = CFString::new(s).0;
        let __CFString(ref key) = *k;
        key
    }
    // Window 
    pub fn GetWindowList (window_id: usize) -> Windows {
        let mut windows: Windows = Vec::new();

        unsafe{
            let windowList: CFArrayRef = CGWindowListCopyWindowInfo(
                kCGWindowListExcludeDesktopElements, 
                // window_id as CGWindowID
                kCGNullWindowID
            );
            let length = CFArrayGetCount(windowList) as usize;

            // let window_list = CFArray(windowList);
            // for i in 0..window_list.len(){
            //     let _wd = window_list.get(i as CFIndex);
            //     let window_dict = CFDictionary(_wd);
            // }

            // from Quartz import CGWindowListCopyWindowInfo, kCGWindowListExcludeDesktopElements, kCGNullWindowID
            // from Quartz import CGGetDisplaysWithRect, CGRect
            // map( lambda w: int(w['kCGWindowNumber']), list(CGWindowListCopyWindowInfo(kCGWindowListExcludeDesktopElements, kCGNullWindowID)) )
            for i in 0..length {
                let idx: CFIndex = i as CFIndex;
                let _wd: CFDictionaryRef = CFArrayGetValueAtIndex(windowList, idx); // CFDictionaryRef
                let cf_dict = CFDictionary(_wd); // methods: find, get, kvs (unknow error)
                // https://developer.apple.com/library/mac/documentation/Carbon/Reference/CGWindow_Reference/#//apple_ref/doc/constant_group/Required_Window_List_Keys
                // Required keys:
                // const CFStringRef kCGWindowNumber;        encoded as kCGWindowIDCFNumberType -> kCFNumberSInt32Type -> i32
                // const CFStringRef kCGWindowStoreType;     encoded as kCFNumberIntType -> libc::c_int -> i32
                // const CFStringRef kCGWindowLayer;         encoded as kCFNumberIntType
                // const CFStringRef kCGWindowBounds;        CFDictionaryRef
                // const CFStringRef kCGWindowSharingState;  encoded as kCFNumberIntType
                // const CFStringRef kCGWindowAlpha;         encoded as kCFNumberFloatType -> libc::c_float    -> f32
                // const CFStringRef kCGWindowOwnerPID;      encoded as kCFNumberIntType   
                // const CFStringRef kCGWindowMemoryUsage;   encoded as kCFNumberLongLongType -> libc::c_longlong -> i64

                // Optional keys
                // const CFStringRef kCGWindowWorkspace;                   encoded as kCFNumberIntType type
                // const CFStringRef kCGWindowOwnerName;                   CFStringRef type
                // const CFStringRef kCGWindowName;                        CFStringRef type
                // const CFStringRef kCGWindowIsOnscreen;                  CFBooleanRef type
                // const CFStringRef kCGWindowBackingLocationVideoMemory;  CFBooleanRef type

                // for ii in 0..(mem::size_of_val(&v)) {
                //     let b = v.offset(ii as isize);
                //     print!("{:?} ", b);
                // }

                let number = CFNumber(mem::transmute(
                    cf_dict.find(str_to_cf_dict_key("kCGWindowNumber")).unwrap()
                )).to_isize(kCGWindowIDCFNumberType).unwrap();
                let store_type = CFNumber(mem::transmute(
                    cf_dict.find(str_to_cf_dict_key("kCGWindowStoreType")).unwrap()
                )).to_isize(kCFNumberIntType).unwrap();
                let layer = CFNumber(mem::transmute(
                    cf_dict.find(str_to_cf_dict_key("kCGWindowLayer")).unwrap()
                )).to_isize(kCFNumberIntType).unwrap();

                let sharing_state = CFNumber(mem::transmute(
                    cf_dict.find(str_to_cf_dict_key("kCGWindowSharingState")).unwrap()
                )).to_isize(kCFNumberIntType).unwrap();

                let alpha = 0.0f32;
                // let alpha = cf_dict.find(str_to_cf_dict_key("kCGWindowAlpha")).unwrap();
                // let alpha_ref: CFNumberRef = mem::transmute( alpha );
                // let alpha = match cf_dict.find(str_to_cf_dict_key("kCGWindowAlpha")) {
                //     Some(alpha) => {
                //         let alpha_ref: CFNumberRef = mem::transmute( alpha );
                //         let alpha = CFNumber(alpha_ref);
                //         match alpha.to_f32() {
                //             Some(alpha) => {
                //                 alpha
                //             },
                //             None => {
                //                 0.0f32
                //             }
                //         }
                //     },
                //     None => {
                //         -0.0f32
                //     }
                // };
                // let alpha = CFNumber(mem::transmute(
                //     cf_dict.find(str_to_cf_dict_key("kCGWindowAlpha")).unwrap()
                // )).to_f32().unwrap();

                let owner_pid = CFNumber(mem::transmute(
                    cf_dict.find(str_to_cf_dict_key("kCGWindowOwnerPID")).unwrap()
                )).to_isize(kCFNumberIntType).unwrap();

                let memory_usage = CFNumber(mem::transmute(
                    cf_dict.find(str_to_cf_dict_key("kCGWindowMemoryUsage")).unwrap()
                )).to_isize(kCFNumberLongLongType).unwrap();

                let workspace = match cf_dict.find(str_to_cf_dict_key("kCGWindowWorkspace")) {
                    Some(_wk) => {
                        CFNumber(mem::transmute(_wk)).to_isize(kCFNumberIntType).unwrap()
                    },
                    None      => {
                        -1isize
                    }
                };

                let owner_name = match cf_dict.find(str_to_cf_dict_key("kCGWindowOwnerName")) {
                    Some(owner_name) => {
                        let owner_name_ref: CFStringRef = mem::transmute(owner_name);
                        let c_string = CFStringGetCStringPtr(owner_name_ref, kCFStringEncodingUTF8);
                        if c_string != ptr::null() {
                            str::from_utf8_unchecked(CStr::from_ptr(c_string).to_bytes())
                        } else {
                            ""
                        }
                    },
                    None      => {
                        ""
                    }
                };
                let name = match cf_dict.find(str_to_cf_dict_key("kCGWindowName")) {
                    Some(name) => {
                        let name_ref: CFStringRef = mem::transmute(name);
                        let c_string = CFStringGetCStringPtr(name_ref, kCFStringEncodingUTF8);
                        if c_string != ptr::null() {
                            str::from_utf8_unchecked(CStr::from_ptr(c_string).to_bytes())
                        } else {
                            ""
                        }
                    },
                    None      => {
                        ""
                    }
                };

                let is_on_screen = match cf_dict.find(str_to_cf_dict_key("kCGWindowIsOnscreen")) {
                    Some(is_on_screen) => {
                        let is_on_screen_ref: CFBooleanRef = mem::transmute(is_on_screen);
                        let is_on_screen =  CFBooleanGetValue(is_on_screen_ref);
                        is_on_screen
                    },
                    None => {
                        false
                    }
                };
                let backing_location_video_memory = match cf_dict.find(str_to_cf_dict_key("kCGWindowBackingLocationVideoMemory")) {
                    Some(backing_location_video_memory) => {
                        let backing_location_video_memory_ref: CFBooleanRef = mem::transmute(backing_location_video_memory);
                        let backing_location_video_memory =  CFBooleanGetValue(backing_location_video_memory_ref);
                        backing_location_video_memory
                    },
                    None => {
                        false
                    }
                };
                // kCGWindowBounds
                let (x,y,width,height) = match cf_dict.find(str_to_cf_dict_key("kCGWindowBounds")) {
                    Some(bounds) => {
                        let bounds_ref: CFDictionaryRef = mem::transmute(bounds);
                        // let bounds: CFDictionary = CFDictionary(bounds_ref);
                        let mut rect: CGRect = CGRect { 
                            size: CGSize {width: 0.0 as CGFloat, height: 0.0 as CGFloat }, 
                            origin: CGPoint { x: 0.0 as CGFloat, y: 0.0 as CGFloat  }
                        };
                        if CGRectMakeWithDictionaryRepresentation(bounds_ref, &mut rect) {
                            (rect.origin.x, rect.origin.y, rect.size.width, rect.size.height)
                        } else {
                            (0.0 as CGFloat, 0.0 as CGFloat, 0.0 as CGFloat, 0.0 as CGFloat)
                        }
                    },
                    None => {
                        (0.0 as CGFloat, 0.0 as CGFloat, 0.0 as CGFloat, 0.0 as CGFloat)
                    }
                };

                let window = Window {
                    number: number, store_type: store_type, layer: layer, sharing_state: sharing_state,
                    alpha: alpha, owner_pid: owner_pid, memory_usage: memory_usage,
                    workspace: workspace, owner_name: owner_name.to_string(), name: name.to_string(),
                    is_on_screen: is_on_screen, backing_location_video_memory: backing_location_video_memory,
                    height: height as f64, width: width as f64, x: x as f64, y: y as f64
                };
                windows.push(window);
            }
        }
        windows
    }



    // Screen
    fn GetMainDisplayID () -> usize {
        unsafe{
            CGMainDisplayID() as usize
        }
    }

    fn GetDisplayScreenSize (display_id: usize) -> CGSize {
        unsafe{
            CGDisplayScreenSize(display_id as CGDirectDisplayID)
        }
    }
    fn GetDisplayRotation (display_id: usize) -> f64 {
        unsafe{
            CGDisplayRotation(display_id as CGDirectDisplayID) as f64
        }
    }
    fn DisplayIsMain (display_id: usize) -> bool {
        unsafe{
            let r = CGDisplayIsMain(display_id as CGDirectDisplayID) as usize;
            if r > 0 {
                true
            } else {
                false
            }
        }
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

    pub fn GetActiveDisplayList () -> Vec<usize> {
        unsafe{
            let mut count: CGDisplayCount = 0;
            let mut err: CGError;
            err = CGGetActiveDisplayList(0, 0 as *mut CGDirectDisplayID, &mut count );
            if err != CGErrorSuccess {
                println!("Error getting number of displays.");
                return vec![];
            }
            let mut disps: Vec<CGDisplayCount> = Vec::with_capacity(count as usize);
            disps.set_len(count as usize);
            
            println!("Disps: {:?}", disps );

            err = CGGetActiveDisplayList(disps.len() as CGDisplayCount, &mut disps[0] as *mut CGDirectDisplayID, &mut count);
            if err != CGErrorSuccess {
                println!("Error getting list of displays.");
                return vec![];
            }
            let mut screens: Vec<usize> = Vec::new();
            for i in disps{
                screens.push(i as usize);
            }
            screens
        }
    }
    pub fn test(){
        println!("Test Function.");
    }
}


#[cfg(target_os = "linux")]
pub mod ffi {

    fn GetWindowList (){

    }
    fn GetMainDisplayID () {

    }
    fn GetActiveDisplayList (){

    }
}

#[cfg(target_os = "windows")]
pub mod ffi {
    fn GetActiveDisplayList (){

    }
}



pub fn display_list() -> Vec<usize> {
    ffi::GetActiveDisplayList()
}


pub fn window_list() -> Vec<ffi::Window> {
    ffi::GetWindowList(0usize)
}
