

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
    // extern crate image;
    extern crate bmp;
    extern crate lodepng;

    #[derive(Debug)]
    pub struct Window {
        pub number: isize,
        pub store_type: isize,
        pub layer: isize,
        pub sharing_state: isize,
        pub alpha: f32,
        pub owner_pid: isize,
        pub memory_usage: isize,  // i64
        pub workspace: isize,
        pub owner_name: String, // &'static str
        pub name: String,
        pub is_on_screen: bool,
        pub backing_location_video_memory: bool,
        // kCGWindowBounds
        pub height: f64,
        pub width: f64,
        pub x: f64,
        pub y: f64,
    }
    pub type Windows = Vec<Window>;
    #[derive(Debug)]
    pub struct Image {
        pub data: Vec<u8>,
        pub height: usize,
        pub width: usize,
        // pub row_len: usize, // Might be superfluous
        pub pixel_width: usize,
    }
    
    impl Image {
        pub fn to_png(&self, path: &str) {
            // let d = lodepng::encode_memory(image_buff, width as usize, height as usize, lodepng::ffi::ColorType::LCT_RGBA, 8).unwrap();
            // println!("{:?}", d.as_cslice().as_ref() );
            
            let image_buff = self.data.as_slice();

            let res = lodepng::encode_file(path, image_buff, 
                                self.width as usize, self.height as usize, 
                                lodepng::ffi::ColorType::LCT_RGBA, 8);
            match res {
                Ok(_) => {
                    // println!("{:?}", );
                },
                Err(e) => {
                    // Debug.
                    println!("Image Path: {:?}", path);
                    println!("Width: {:?} Height: {:?} ", self.width, self.height );
                    println!("Bytes: {:?}", image_buff.len() );
                    println!("Error: {:?}",  e);
                }
            }
        }
        pub fn to_bmp(&self, path: &str) {
            // let mut im = bmp::Image::new(self.width as u32, self.height as u32);
            let image_buff = self.data.as_slice();

            // for row in (0..(self.height)) {
            //     for col in ((0..self.width)) {
            //         let idx = row * (self.width * self.pixel_width) + col * self.pixel_width;
            //         let b = image_buff[idx];
            //         let g = image_buff[idx] + 1;
            //         let r = image_buff[idx] + 2;
            //         let a = image_buff[idx] + 3;
            //         im.set_pixel( col as u32, row as u32, bmp::Pixel {r: r, g: g, b: b});
            //     }
            // }
            let mut pixels: Vec<Vec<Vec<u8>>> = Vec::with_capacity(self.height);
            for row in (0..self.height) {
                let mut line: Vec<Vec<u8>> = Vec::with_capacity(self.width);
                for col in (0..self.width) {
                    let idx = row*self.width*self.pixel_width + col*self.pixel_width;
                    let mut pixel: Vec<u8> = Vec::with_capacity(self.pixel_width);
                    for i in (idx..(idx+self.pixel_width)) {
                        // B, G, R, A
                        pixel.push(image_buff[i]);
                    }
                    line.push(pixel);
                }
                pixels.push(line);
            }
            // BMP Format
            let mut im = bmp::Image::new(self.width as u32, self.height as u32);
            for row in (0..self.height) {
                for col in (0..self.width) {
                    im.set_pixel( col as u32, row as u32,
                        bmp::Pixel {r: pixels[row][col][2], g: pixels[row][col][1], b: pixels[row][col][0]}
                    );
                }
            }
            let res = im.save(path);
            match res {
                Ok(_) => {
                    // pass
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        }
        pub fn get_raw_data(&self) {
            // self.data.as_slice()
        }
    }
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
    use self::core_foundation::base:: { CFRange, Boolean, CFRelease };

    use self::core_foundation::dictionary::{ 
        CFDictionaryContainsKey, CFDictionaryGetCount, CFDictionaryGetValueIfPresent, 
        CFDictionaryGetKeysAndValues
    };
    use self::core_foundation::string::{ 
        CFString, CFStringRef, __CFString, 
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

    type CGImageRef        = *mut u8; // *mut CGImage
    type CGDataProviderRef = *mut u8; // *mut CGDataProvider


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

        fn CGDataProviderCopyData(provider: CGDataProviderRef) -> CFDataRef;
    }
    unsafe fn str_to_cf_dict_key (s: &str) -> &libc::c_void {
        let k = CFString::new(s).0;
        let __CFString(ref key) = *k;
        key
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
    pub fn GetWindowList (window_id: usize) -> Windows {
        let mut windows: Windows = Vec::new();

        unsafe{
            let windowList: CFArrayRef = CGWindowListCopyWindowInfo(
                kCGWindowListExcludeDesktopElements, 
                window_id as CGWindowID
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

                let window = Window {
                    number: number, store_type: store_type, layer: layer, sharing_state: sharing_state,
                    alpha: alpha, owner_pid: owner_pid, memory_usage: memory_usage,
                    workspace: workspace, owner_name: owner_name, name: name,
                    is_on_screen: is_on_screen, 
                    backing_location_video_memory: backing_location_video_memory,
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
    pub unsafe fn DisplayCreateImage (display_id: usize) -> Result<Image, &'static str> {
        let display_image = CGDisplayCreateImage(display_id as CGDirectDisplayID);
        // Get info about image
        let width = CGImageGetWidth(display_image) as usize;
        let height = CGImageGetHeight(display_image) as usize;
        // let row_len = CGImageGetBytesPerRow(display_image) as usize;
        let pixel_bits = CGImageGetBitsPerPixel(display_image) as usize;
        if pixel_bits % 8 != 0 {
            return Err("Pixels aren't integral bytes.")
        }

        // Copy image into a Vec buffer
        let cf_data = CGDataProviderCopyData(CGImageGetDataProvider(display_image));
        let raw_len = CFDataGetLength(cf_data) as usize;

        let res = if width*height*pixel_bits != raw_len*8 {
            Err("Image size is inconsistent with W*H*D.")
        } else {
            let data = slice::from_raw_parts(CFDataGetBytePtr(cf_data), raw_len).to_vec();
            Ok(Image {
                data: data,
                height: height,
                width: width,
                // row_len: row_len,
                pixel_width: pixel_bits/8
            })
        };

        // Release native objects
        CGImageRelease(display_image);
        CFRelease(cf_data as *const libc::c_void);
        res
    }

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

            err = CGGetActiveDisplayList(
                    disps.len() as CGDisplayCount, 
                    &mut disps[0] as *mut CGDirectDisplayID, 
                    &mut count );
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




#[cfg(target_os = "windows")]
pub mod ffi {
#![allow(non_snake_case, dead_code)]

    use libc::{c_int, c_uint, c_long, c_void};
    use std::intrinsics::{size_of};

    use ::Screenshot;
    use ::ScreenResult;

    type PVOID = *mut c_void;
    type LPVOID = *mut c_void;
    type WORD = u16; // c_uint;
    type DWORD = u32; // c_ulong;
    type BOOL = c_int;
    type BYTE = u8;
    type UINT = c_uint;
    type LONG = c_long;
    type LPARAM = c_long;

    #[repr(C)]
    struct RECT {
        left: LONG,
        top: LONG,
        right: LONG, // immediately outside rect
        bottom: LONG, // immediately outside rect
    }
    type LPCRECT = *const RECT;
    type LPRECT = *mut RECT;

    type HANDLE = PVOID;
    type HMONITOR = HANDLE;
    type HWND = HANDLE;
    type HDC = HANDLE;
    #[repr(C)]
    struct MONITORINFO {
        cbSize: DWORD,
        rcMonitor: RECT,
        rcWork: RECT,
        dwFlags: DWORD,
    }
    type LPMONITORINFO = *mut MONITORINFO;
    type MONITORENUMPROC = fn(HMONITOR, HDC, LPRECT, LPARAM) -> BOOL;

    type HBITMAP = HANDLE;
    type HGDIOBJ = HANDLE;
    type LPBITMAPINFO = PVOID; // Hack

    const NULL: *mut c_void = 0usize as *mut c_void;
    const HGDI_ERROR: *mut c_void = -1isize as *mut c_void;
    const SM_CXSCREEN: c_int = 0;
    const SM_CYSCREEN: c_int = 1;

    /// TODO verify value
    const SRCCOPY: u32 = 0x00CC0020;
    const CAPTUREBLT: u32 = 0x40000000;
    const DIB_RGB_COLORS: UINT = 0;
    const BI_RGB: DWORD = 0;

    #[repr(C)]
    struct BITMAPINFOHEADER {
        biSize: DWORD,
        biWidth: LONG,
        biHeight: LONG,
        biPlanes: WORD,
        biBitCount: WORD,
        biCompression: DWORD,
        biSizeImage: DWORD,
        biXPelsPerMeter: LONG,
        biYPelsPerMeter: LONG,
        biClrUsed: DWORD,
        biClrImportant: DWORD,
    }

    #[repr(C)]
    struct RGBQUAD {
        rgbBlue: BYTE,
        rgbGreen: BYTE,
        rgbRed: BYTE,
        rgbReserved: BYTE,
    }

    /// WARNING variable sized struct
    #[repr(C)]
    struct BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER,
        bmiColors: [RGBQUAD; 1],
    }
    #[derive(Debug)]
    pub struct Image {
        pub data: Vec<u8>,
        pub height: usize,
        pub width: usize,
        // pub row_len: usize, // Might be superfluous
        pub pixel_width: usize,
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetSystemMetrics(m: c_int) -> c_int;
        fn EnumDisplayMonitors(hdc: HDC, lprcClip: LPCRECT,
                               lpfnEnum: MONITORENUMPROC, dwData: LPARAM) -> BOOL;
        fn GetMonitorInfo(hMonitor: HMONITOR, lpmi: LPMONITORINFO) -> BOOL;
        fn GetDesktopWindow() -> HWND;
        fn GetDC(hWnd: HWND) -> HDC;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateCompatibleDC(hdc: HDC) -> HDC;
        fn CreateCompatibleBitmap(hdc: HDC, nWidth: c_int, nHeight: c_int) -> HBITMAP;
        fn SelectObject(hdc: HDC, hgdiobj: HGDIOBJ) -> HGDIOBJ;
        fn BitBlt(hdcDest: HDC, nXDest: c_int, nYDest: c_int, nWidth: c_int, nHeight: c_int,
                  hdcSrc: HDC, nXSrc: c_int, nYSrc: c_int, dwRop: DWORD) -> BOOL;
        fn GetDIBits(hdc: HDC, hbmp: HBITMAP, uStartScan: UINT, cScanLines: UINT,
                     lpvBits: LPVOID, lpbi: LPBITMAPINFO, uUsage: UINT) -> c_int;

        fn DeleteObject(hObject: HGDIOBJ) -> BOOL;
        fn ReleaseDC(hWnd: HWND, hDC: HDC) -> c_int;
        fn DeleteDC(hdc: HDC) -> BOOL;
    }

    /// Reorder rows in bitmap, last to first.
    /// TODO rewrite functionally
    fn flip_rows(data: Vec<u8>, height: usize, row_len: usize) -> Vec<u8> {
        let mut new_data = Vec::with_capacity(data.len());
        unsafe {new_data.set_len(data.len())};
        for row_i in (0..height) {
            for byte_i in (0..row_len) {
                let old_idx = (height-row_i-1)*row_len + byte_i;
                let new_idx = row_i*row_len + byte_i;
                new_data[new_idx] = data[old_idx];
            }
        }
        new_data
    }

    /// TODO Support multiple screens
    /// This may never happen, given the horrific quality of Win32 APIs
    pub fn DisplayCreateImage(display_id: usize) -> Result<Image, &'static str> {
        unsafe {
            // Enumerate monitors, getting a handle and DC for requested monitor.
            // loljk, because doing that on Windows is worse than death
            let h_wnd_screen = GetDesktopWindow();
            let h_dc_screen = GetDC(h_wnd_screen);
            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);

            // Create a Windows Bitmap, and copy the bits into it
            let h_dc = CreateCompatibleDC(h_dc_screen);
            if h_dc == NULL { return Err("Can't get a Windows display.");}

            let h_bmp = CreateCompatibleBitmap(h_dc_screen, width, height);
            if h_bmp == NULL { return Err("Can't create a Windows buffer");}

            let res = SelectObject(h_dc, h_bmp);
            if res == NULL || res == HGDI_ERROR {
                return Err("Can't select Windows buffer.");
            }

            let res = BitBlt(h_dc, 0, 0, width, height, h_dc_screen, 0, 0, SRCCOPY|CAPTUREBLT);
            if res == 0 { return Err("Failed to copy screen to Windows buffer");}

            // Get image info
            let pixel_width: usize = 4; // FIXME
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as DWORD,
                    biWidth: width as LONG,
                    biHeight: height as LONG,
                    biPlanes: 1,
                    biBitCount: 8*pixel_width as WORD,
                    biCompression: BI_RGB,
                    biSizeImage: (width * height * pixel_width as c_int) as DWORD,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD {
                    rgbBlue: 0,
                    rgbGreen: 0,
                    rgbRed: 0,
                    rgbReserved: 0
                }],
            };

            // Create a Vec for image
            let size: usize = (width*height) as usize * pixel_width;
            let mut data: Vec<u8> = Vec::with_capacity(size);
            data.set_len(size);

            // copy bits into Vec
            GetDIBits(h_dc, h_bmp, 0, height as DWORD,
                &mut data[0] as *mut u8 as *mut c_void,
                &mut bmi as *mut BITMAPINFO as *mut c_void,
                DIB_RGB_COLORS);

            // Release native image buffers
            ReleaseDC(h_wnd_screen, h_dc_screen); // don't need screen anymore
            DeleteDC(h_dc);
            DeleteObject(h_bmp);

            let data = flip_rows(data, height as usize, width as usize*pixel_width);

            Ok(Image {
                data: data,
                height: height as usize,
                width: width as usize,
                // row_len: width as usize*pixel_width,
                pixel_width: pixel_width,
            })
        }
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







pub fn display_list() -> Vec<usize> {
    ffi::GetActiveDisplayList()
}


pub fn window_list(display_id: usize) -> Vec<ffi::Window> {
    ffi::GetWindowList(display_id)
}

pub fn display_capture (display_id: usize) -> Result<ffi::Image, &'static str> {
    unsafe{
        ffi::DisplayCreateImage(display_id)
    }
}
pub fn window_capture () -> (){

}
pub fn rect_capture () -> (){

}
pub fn display_stream () -> (){

}
pub fn window_stream() -> () {

}


