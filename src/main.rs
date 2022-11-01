#![forbid(unsafe_code)]

// Update vector tile module to new protobuf version
fn main() {
    protobuf_codegen::Codegen::new()
        .out_dir("src/")
        .inputs(&["protos/vector_tile.proto"])
        .include("protos")
        .run()
        .expect("Codegen failed");
}
