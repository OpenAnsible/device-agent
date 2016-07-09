extern crate gcc;

fn main() {
    // ensure you have already install ffmpeg (with static lib).
    gcc::Config::new()
        .file("src/c/avtools.c")
        .include("src")
        .compile("libavtools.a");
}