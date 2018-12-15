use std::env;
use std::path::PathBuf;

use bindgen::Builder as Bindgen;
use cbindgen::Builder as CBindgen;
use glob::glob;

fn main(){
  let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

  let mut bindgen = Bindgen::default();

  for header in glob(&format!("{}/src/*.h", crate_dir)).unwrap() {
    bindgen = bindgen.header(header.unwrap().to_str().unwrap());
  }

  bindgen
    .clang_arg(format!("-I{}/src", crate_dir))
    .generate().unwrap()
    .write_to_file(out_path.join("bindings.rs")).unwrap();

  CBindgen::new()
    .with_crate(crate_dir)
    .generate().unwrap()
    .write_to_file(out_path.join("bindings.h"));
}
