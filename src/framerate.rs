use raylib::prelude::*;
use std::ffi::{CStr, CString};
use std::fs::{self, *};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::{archives, cstr};

fn ffmpeg_folder_convert(from: &str, format: &str) {
    if Path::new(from).is_dir() {
        let mut files: Vec<_> = fs::read_dir(from)
            .unwrap()
            .map(|f| f.unwrap().path())
            .collect();

        files.sort();

        let mut ffmpeg = Command::new("ffmpeg")
            .args(["-framerate", "2", "-i", "-"])
            .args(["out.mp4"])
            .stdin(Stdio::piped())
            .spawn()
            .expect("Cannot spawn ffmpeg command");

        let stdin = ffmpeg.stdin.as_mut().unwrap();

        for path in files {
            let file = fs::read(path).expect("Cannot read file");
            stdin.write_all(file.as_slice());
        }

        stdin.flush().expect("Cannot flush ffmpeg stdin");
        ffmpeg.wait();
    }
}

#[test]
fn convert_png_mp4() {
    ffmpeg_folder_convert("tests/generated", "mp4");
}
