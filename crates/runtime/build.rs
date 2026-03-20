// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Build script for protobuf code generation.
//!
//! This script compiles proto files when the `grpc` feature is enabled:
//! - context.proto: Low-level ledger client (client only)
//! - converge.proto: High-level Converge protocol (server + client)

fn main() {
    #[cfg(feature = "grpc")]
    {
        // Compile context.proto - ledger client (no server needed)
        tonic_build::configure()
            .build_server(false)
            .build_client(true)
            .out_dir("src/ledger/generated")
            .compile(&["proto/context.proto"], &["proto"])
            .expect("Failed to compile context.proto");

        // Compile converge.proto - main Converge protocol (server + client)
        // This is the API for mobile and CLI clients.
        tonic_build::configure()
            .build_server(true)
            .build_client(true)
            .out_dir("src/grpc/generated")
            .compile(&["proto/converge.proto"], &["proto"])
            .expect("Failed to compile converge.proto");
    }
}
