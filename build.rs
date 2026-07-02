use {protoc_bin_vendored::protoc_bin_path, std::env};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc_path = protoc_bin_path()?;
    unsafe {
        env::set_var("PROTOC", protoc_path);
    }

    println!("cargo:rerun-if-changed=proto/aperture/txstream.proto");

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&["proto/aperture/txstream.proto"], &["proto"])?;

    Ok(())
}
