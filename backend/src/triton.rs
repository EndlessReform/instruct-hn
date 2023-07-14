pub mod triton {
    // The string specified here should match the package name in your .proto file, NOT the filename
    tonic::include_proto!("inference");
}
