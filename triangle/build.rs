// build.rs — lives at the crate root, next to Cargo.toml.
// Cargo compiles and runs this BEFORE building your crate.
use std::{env, path::PathBuf, process::Command};

fn main() {
  let shader = "shaders/shader.slang";

  // Re-run this script only when the shader source changes, not on every build.
  println!("cargo::rerun-if-changed={shader}");

  // OUT_DIR is a per-build scratch dir Cargo gives us. Writing the .spv here
  // keeps generated artifacts out of your source tree (so nothing to gitignore).
  let out_dir = env::var("OUT_DIR").unwrap();
  let spv = PathBuf::from(&out_dir).join("shader.spv");

  let status = Command::new("slangc")
    .args([shader, "-target", "spirv", "-o"])
    .arg(&spv)
    .status()
    .expect("failed to run slangc — is it on PATH?");

  // Treat a shader compile failure as a build failure.
  assert!(status.success(), "slangc failed to compile {shader}");
}
