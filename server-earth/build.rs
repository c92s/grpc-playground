fn main() {
    tonic_build::compile_protos("proto/earth.proto").unwrap();
}
