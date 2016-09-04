#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, unused_must_use, non_snake_case)]

pub use std::convert::AsRef;
pub use std::time::{ Duration, SystemTime };
pub use std::thread;

pub use self::ffi::CGImage;

#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, unused_must_use, non_snake_case)]
#[derive(Debug, Clone)]
pub struct Image {
    pub data: Vec<u8>,     // BGRA
    pub height: usize,
    pub width: usize,
    // pub row_len: usize, // Might be superfluous
    pub pixel_width: usize,
}

#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, unused_must_use, non_snake_case)]
#[cfg(target_os = "macos")]
pub mod ffi {
    extern crate core_foundation;
    extern crate core_graphics;

    use std::string::ToString;
    use std::boxed::Box;
    use std::sync::Arc;
    use super::{
        thread, AsRef, SystemTime, Duration,
        Image, Window, Screen,
        // bgr_to_yuv, ycbcr_to_rgb, rgb_to_ycbcr
    };
    
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
    use self::core_graphics::geometry::{ CGSize, CGPoint, CGRect };
    use self::core_graphics::base::{ CGFloat };
    use self::core_graphics::color_space::{ CGColorSpaceRef, CGColorSpace, ColorSpace };
    pub use self::core_graphics::image::{ CGImageRef, CGImage };
    use self::core_graphics::data_provider::{ CGDataProviderRef, CGDataProvider };

    use self::core_foundation::base::{ CFRange, Boolean, CFRelease };

    use self::core_foundation::dictionary::{ 
        CFDictionaryContainsKey, CFDictionaryGetCount, CFDictionaryGetValueIfPresent, 
        CFDictionaryGetKeysAndValues
    };
    use self::core_foundation::string::{ 
        CFString, CFStringRef, 
        CFStringGetCStringPtr, CFStringGetLength, CFStringGetCString, CFStringGetBytes, 
        kCFStringEncodingUTF8 
    };
    use self::core_foundation::number::{ 
        CFNumber, CFNumberRef, __CFNumber, CFNumberType, CFNumberGetValue,
        kCFNumberSInt32Type, kCGWindowIDCFNumberType, kCFNumberSInt64Type, kCFNumberFloat32Type, 
        kCFNumberFloat64Type, kCFNumberIntType, kCFNumberLongType, kCFNumberLongLongType, 
        kCFNumberFloatType, CFBooleanGetValue
    };
    use self::core_foundation::boolean:: {
        CFBoolean, CFBooleanRef, kCFBooleanTrue, kCFBooleanFalse
    };
    use self::core_foundation::data:: { CFDataRef, __CFData, CFDataGetBytePtr, CFDataGetLength };

    use std::ffi::{ CString, CStr };
    use std::{ ptr, mem, str, slice };

    type CGDisplayCount = libc::uint32_t;

    const CGErrorSuccess: CGError = 0;
    const CGErrorFailure: CGError = 1000;

    extern {
        pub fn CFArrayGetValueAtIndex(theArray: CFArrayRef, idx: CFIndex) -> CFDictionaryRef;
        pub fn CGRectMakeWithDictionaryRepresentation(dict: CFDictionaryRef, rect: *mut CGRect ) -> bool;

        fn CGDisplayCreateImage(displayID: CGDirectDisplayID) -> CGImageRef;
        
        fn CGImageRelease(image: CGImageRef);

        fn CGImageGetBitsPerComponent(image: CGImageRef) -> libc::size_t;
        fn CGImageGetBitsPerPixel(image: CGImageRef) -> libc::size_t;
        fn CGImageGetBytesPerRow(image: CGImageRef) -> libc::size_t;
        fn CGImageGetDataProvider(image: CGImageRef) -> CGDataProviderRef;
        fn CGImageGetHeight(image: CGImageRef) -> libc::size_t;
        fn CGImageGetWidth (image: CGImageRef) -> libc::size_t;
        fn CGImageCreateCopyWithColorSpace(image: CGImageRef, space: CGColorSpaceRef) -> CGImageRef;
        fn CGDataProviderCopyData(provider: CGDataProviderRef) -> CFDataRef;
    }
    unsafe fn str_to_cf_dict_key (s: &str) -> CFStringRef {
        CFString::new(s).0
    }
    unsafe fn cf_string_ref_to_string (theString: CFStringRef) -> String {
        let c_string = CFStringGetCStringPtr(theString, kCFStringEncodingUTF8);
        if c_string != ptr::null() {
            str::from_utf8_unchecked(CStr::from_ptr(c_string).to_bytes()).to_string()
        } else {
            let char_len: CFIndex = CFStringGetLength(theString);

            // First, ask how big the buffer ought to be.
            let mut bytes_required: CFIndex = 0;
            CFStringGetBytes(theString,
                             CFRange { location: 0, length: char_len },
                             kCFStringEncodingUTF8,
                             0,
                             false as Boolean,
                             ptr::null_mut(),
                             0,
                             &mut bytes_required);

            // Then, allocate the buffer and actually copy.
            let mut buffer = vec![b'\x00'; bytes_required as usize];

            let mut bytes_used: CFIndex = 0;
            let chars_written = CFStringGetBytes(theString,
                                                 CFRange { location: 0, length: char_len },
                                                 kCFStringEncodingUTF8,
                                                 0,
                                                 false as Boolean,
                                                 buffer.as_mut_ptr(),
                                                 buffer.len() as CFIndex,
                                                 &mut bytes_used) as usize;
            assert!(chars_written as CFIndex == char_len);

            // This is dangerous; we over-allocate and null-terminate the string (during
            // initialization).
            assert!(bytes_used == buffer.len() as CFIndex);
            str::from_utf8_unchecked(&buffer).to_string()
        }
    }
    // Window 
    pub fn GetWindowList (window_id: &usize) -> Vec<Window> {
        let mut windows: Vec<Window> = Vec::new();

        unsafe{
            let windowList: CFArrayRef = CGWindowListCopyWindowInfo(
                kCGWindowListExcludeDesktopElements, 
                window_id.clone() as CGWindowID
                // kCGNullWindowID
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
                // https://developer.apple.com/library/mac/documentation/Carbon/Reference/CGWindow_Reference
                // /#//apple_ref/doc/constant_group/Required_Window_List_Keys
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
                        cf_string_ref_to_string(owner_name_ref)
                    },
                    None      => {
                        "".to_string()
                    }
                };
                let name = match cf_dict.find(str_to_cf_dict_key("kCGWindowName")) {
                    Some(name) => {
                        let name_ref: CFStringRef = mem::transmute(name);
                        cf_string_ref_to_string(name_ref)
                    },
                    None      => {
                        "".to_string()
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
                let window = Window::new(
                                    number as usize, name, owner_name, owner_pid as usize,
                                    memory_usage as usize, Some(alpha), Some(workspace),
                                    is_on_screen, width as f64, height as f64, 
                                    x as f64, y as f64
                );
                windows.push(window);
            }
        }
        windows
    }
    // Screen
    pub fn GetMainDisplayID () -> usize {
        unsafe{
            CGMainDisplayID() as usize
        }
    }

    pub fn GetDisplayScreenSize (display_id: &usize) -> CGSize {
        unsafe{
            CGDisplayScreenSize(display_id.clone() as CGDirectDisplayID)
        }
    }
    pub fn GetDisplayRotation (display_id: &usize) -> f64 {
        unsafe{
            CGDisplayRotation(display_id.clone() as CGDirectDisplayID) as f64
        }
    }
    pub fn DisplayIsMain (display_id: &usize) -> bool {
        unsafe{
            let r = CGDisplayIsMain(display_id.clone() as CGDirectDisplayID) as usize;
            r > 0
        }
    }
    pub unsafe fn DisplayCreateImage ( display_id: &usize, )-> Result<CGImage, &'static str> {
        let display_image: CGImageRef = CGDisplayCreateImage(display_id.clone() as CGDirectDisplayID);
        let image: CGImage            = CGImage::from_ref(display_image);
        Ok(image)
    }
    pub fn GetActiveDisplayList () -> Vec<Screen> {
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

            err = CGGetActiveDisplayList(
                    disps.len() as CGDisplayCount, 
                    &mut disps[0] as *mut CGDirectDisplayID, 
                    &mut count );
            if err != CGErrorSuccess {
                println!("Error getting list of displays.");
                return vec![];
            }
            let mut screens: Vec<Screen> = Vec::new();
            for i in disps{
                screens.push(Screen::new(i as usize));
            }
            screens
        }
    }
    pub fn test(){
        println!("Test Function.");
    }
}

#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, unused_must_use, non_snake_case)]
#[derive(Debug)]
pub struct Window {
    window_id: usize, // number
    name: String,
    owner_name: String,
    owner_pid : usize,
    memory_usage: usize,
    alpha: Option<f32>,
    workspace: Option<isize>,
    is_on_screen: bool,
    width : f64,
    height: f64,
    x: f64,
    y: f64
}

#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, unused_must_use, non_snake_case)]
#[derive(Debug)]
pub struct Screen {
    display_id: usize,
    // size
}

#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, unused_must_use, non_snake_case)]
impl Window {
    pub fn new(window_id: usize, name: String, owner_name: String, owner_pid: usize,
                memory_usage: usize, alpha: Option<f32>, workspace: Option<isize>,
                is_on_screen: bool, width: f64, height: f64, x: f64, y: f64)
                -> Window {
        Window {
            window_id : window_id,
            name      : name,
            owner_name: owner_name,
            owner_pid : owner_pid,
            memory_usage: memory_usage,
            alpha       : alpha,
            workspace   : workspace,
            is_on_screen: is_on_screen,
            width : width,
            height: height,
            x     : x,
            y     : y
        }
    }
    pub fn id (&self) -> usize {
        self.window_id
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn owner_name(&self) -> String {
        self.owner_name.clone()
    }
    pub fn pid(&self) -> usize {
        self.owner_pid()
    }
    pub fn owner_pid(&self) -> usize {
        self.owner_pid
    }
    pub fn width(&self) -> f64 {
        self.width
    }
    pub fn height(&self) -> f64 {
        self.height
    }
    pub fn x(&self) -> f64 {
        self.x
    }
    pub fn y(&self) -> f64 {
        self.y
    }
    pub fn is_on_screen(&self) -> bool {
        self.is_on_screen
    }
    pub fn workspace(&self) -> Option<isize> {
        self.workspace
    }
    pub fn alpha(&self) -> Option<f32> {
        self.alpha
    }

    pub fn children(&self) -> Option<Vec<Window>> {
        None
    }
    pub fn parent(&self) -> Option<Window> {
        None
    }

    pub fn capture(&self) -> () {

    }
    pub fn capture_with_rect(&self) -> () {

    }
    pub fn record(&self) -> () {

    }
    pub fn record_with_rect(&self) -> () {

    }

}
#[allow(unused_imports, unused_unsafe, unused_variables, unused_assignments, non_upper_case_globals, dead_code, improper_ctypes, unreachable_code, unused_must_use, non_snake_case)]
impl Screen {
    pub fn new(display_id: usize) -> Screen {
        Screen { display_id: display_id }
    }
    pub fn main() -> Screen {
        // Main Display
        unsafe{
            Screen::new(ffi::GetMainDisplayID())
        }
    }
    pub fn list () -> Vec<Screen> {
        // Display List
        unsafe{
            ffi::GetActiveDisplayList()
        }
    }
    pub fn id(&self) -> usize {
        self.display_id
    }
    pub fn windows (&self) -> Vec<Window>{
        unsafe {
            ffi::GetWindowList(&self.display_id)
        }
    }
    pub fn is_main(&self) -> bool {
        unsafe {
            ffi::DisplayIsMain(&self.display_id)
        }
    }
    pub fn rotation(&self) -> f64 {
        unsafe {
            ffi::GetDisplayRotation(&self.display_id)
        }
    }
    pub fn size(&self) -> (usize, usize){
        unsafe {
            let cg_size = ffi::GetDisplayScreenSize(&self.display_id);
            // CGFloat .
            (cg_size.width as usize, cg_size.height as usize)
        }
    }
    pub fn width(&self) -> usize {
        unsafe {
            let cg_size = ffi::GetDisplayScreenSize(&self.display_id);
            // CGFloat .
            cg_size.width as usize
        }
    }
    pub fn height(&self) -> usize {
        unsafe {
            let cg_size = ffi::GetDisplayScreenSize(&self.display_id);
            // CGFloat .
            cg_size.height as usize
        }
    }
    pub fn capture(&self) -> Result<ffi::CGImage, ()> {
        unsafe {
            let image = ffi::DisplayCreateImage(&self.display_id).unwrap();
            Ok(image)
        }
    }
    pub fn capture_with_rect(&self, x: usize, y: usize, 
                             width: usize, height: usize) 
                             ->() {
        ()
    }
    pub fn record(&self) -> () {
        ()
    }
    pub fn record_with_rect(&self, x: usize, y: usize, 
                            width: usize, height: usize)
                            -> () {
        ()
    }
}




pub fn screens () -> Vec<Screen>{
    ffi::GetActiveDisplayList()
}



