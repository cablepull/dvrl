extern crate cc;

use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    println!("Prepare to compile");
    let out_dir = env::var("OUT_DIR").unwrap();

    // Note that there are a number of downsides to this approach, the comments
    // below detail how to improve the portability of these commands.
    Command::new("gcc").args(&["native/process_input.c", "-c", "-fPIC", "","-o"])
                       .arg(&format!("{}/process_input.o", out_dir))
                       .status().unwrap();
    Command::new("gcc").args(&["-shared", "-o", &format!("{}/mypi.so", out_dir), &format!("{}/process_input.o", out_dir),])
                      .current_dir(&Path::new(&out_dir))
                      .status().unwrap();

                      println!("cargo:rustc-link-search=native={}", out_dir);
                      println!("cargo:rustc-link-lib=static=mypi");
                      println!("cargo:rerun-if-changed=native/process_input.c");
                      println!("cargo:rustc-env=OUT_DIR={}", out_dir);
}