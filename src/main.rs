// Update vector tile module to new protobuf version
fn main() {
    protobuf_codegen_pure::run(protobuf_codegen_pure::Args {
        out_dir: "src/",
        input: &["protos/vector_tile.proto"],
        includes: &["protos"],
        customize: protobuf_codegen_pure::Customize {
          ..Default::default()
        },
    }).expect("protoc");
}
