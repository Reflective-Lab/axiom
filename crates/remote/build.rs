// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false) // Client only
        .build_client(true)
        // Don't derive serde for all types - prost_types::Struct doesn't support it
        // We'll handle JSON conversion manually where needed
        .compile_protos(&["proto/converge.proto"], &["proto/"])?;
    Ok(())
}
