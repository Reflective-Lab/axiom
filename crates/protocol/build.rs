// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().build_client(true).build_server(true).compile_protos(
        &[concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../schema/proto/converge.proto"
        )],
        &[concat!(env!("CARGO_MANIFEST_DIR"), "/../../schema/proto/")],
    )?;

    Ok(())
}
