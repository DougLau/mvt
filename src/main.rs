#![forbid(unsafe_code)]

// Update vector tile module to new protobuf version
fn main() {
    prost_build::Config::new()
        .out_dir("src")
        .compile_protos(&["protos/vector_tile.proto"], &["protos"])
        .expect("Codegen failed");
}
