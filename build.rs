extern crate gcc;

fn main() {
    // ensure you have already install ffmpeg (with static lib).
    gcc::Config::new()
        .file("libavtools/avtools.c")
        .include("src")
        .compile("libavtools.a");
}