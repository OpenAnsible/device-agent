mod ffi;

// #[derive(Debug)]
// pub struct Screen {
//     pub id     : usize,
//     pub windows: Windows
// }

// #[derive(Debug)]
// pub struct Window {
//     id    : usize,
//     title : String,
//     x     : usize,
//     y     : usize,
//     width : usize,
//     height: usize
// }

// pub type Windows = Vec<Window>;

pub mod display {
    pub fn list () -> Vec<usize> {
        // GetDisplayList

        // let mut screens: Vec<Screen> = Vec::new();
        // ffi::api::GetActiveDisplayList()

        Vec::new()

        // let window: Window = Window { id: 1, title: String::from("测试"), x: 0, y: 0, width: 1440, height: 900 };
        // let windows: Vec<Window> = vec![window];
        // screens.push(Screen { id : 123456, windows: vec![window] });
        // screens
    }
}

pub mod window {
    pub fn list () {
        
    }
}

pub fn hello () {
    println!("hello, world!");
}

pub fn main (){
    println!("this is main at screen.rs");
}

