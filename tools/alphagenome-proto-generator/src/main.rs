use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let proto_root = PathBuf::from(args.next().ok_or("missing proto root")?);
    let output_root = PathBuf::from(args.next().ok_or("missing output root")?);
    if args.next().is_some() {
        return Err("unexpected extra argument".into());
    }

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .out_dir(output_root)
        .compile_protos(
            &[proto_root.join("dna_model_service.proto")],
            &[proto_root],
        )?;
    Ok(())
}
