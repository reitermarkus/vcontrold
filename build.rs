use std::env;
use std::path::PathBuf;
use std::fs::File;

use bindgen::Builder as Bindgen;
use cbindgen::{Builder as CBindgen, Language::C};
use glob::glob;

const INCLUDE_GUARD: &str = "LIBVCONTROL_H";

fn main(){
  if cfg!(not(target_os = "macos")) {
    println!("cargo:rustc-link-lib=dl");
  }

  let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
  let out_dir = env::var("OUT_DIR").unwrap();
  let out_path = PathBuf::from(&out_dir);
  let target_path = out_path.parent().unwrap().parent().unwrap().parent().unwrap();

  let mut bindgen = Bindgen::default();

  for header in glob(&format!("{}/src/*.h", crate_dir)).unwrap() {
    bindgen = bindgen.header(header.unwrap().to_str().unwrap());
  }

  File::create(out_path.join("bindings.h")).unwrap();

  bindgen
    .clang_arg(format!("-I{}/src", crate_dir))
    .clang_arg(format!("-I{}", out_dir))
    .clang_arg(format!("-D{}", INCLUDE_GUARD))
    .generate().unwrap()
    .write_to_file(out_path.join("bindings.rs")).unwrap();

  CBindgen::new()
    .with_include_guard(INCLUDE_GUARD)
    .with_language(C)
    .with_crate(crate_dir)
    .generate().unwrap()
    .write_to_file(target_path.join("bindings.h"));
}
