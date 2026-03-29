// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Local Inference — run LLM inference on Apple Silicon.
//!
//! See README.md for feature flags and expected performance.

use converge_llm::{InferenceEnvelope, PromptStackBuilder, StateInjection, UserIntent};

#[cfg(feature = "wgpu")]
type Backend = burn::backend::Wgpu;

#[cfg(all(feature = "ndarray", not(feature = "wgpu")))]
type Backend = burn::backend::NdArray;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Converge Local Inference ===\n");

    // Build a prompt using the contract system
    let stack = PromptStackBuilder::new()
        .state(
            StateInjection::new()
                .with_scalar("mae", 0.12)
                .with_scalar("success_ratio", 0.88)
                .with_list(
                    "top_features",
                    vec!["user_engagement".into(), "session_duration".into()],
                ),
        )
        .intent(
            UserIntent::new("analyze_model_performance")
                .with_criteria("identify areas for improvement"),
        )
        .build();

    let rendered = stack.render();
    println!("Prompt ({} chars):", rendered.len());
    println!("{}\n", &rendered[..rendered.len().min(300)]);

    // Create deterministic envelope for reproducibility
    let envelope = InferenceEnvelope::deterministic("local:v1", 42);
    println!(
        "Envelope: deterministic={}, max_tokens={}\n",
        envelope.is_deterministic(),
        envelope.stopping.max_tokens
    );

    run_inference(&stack, &envelope);
}

#[cfg(all(feature = "tiny", feature = "pretrained"))]
fn run_inference(
    stack: &converge_llm::PromptStack,
    envelope: &converge_llm::InferenceEnvelope,
) {
    use converge_llm::TinyLlamaEngine;
    use std::time::Instant;

    println!("Loading TinyLlama 1.1B...");
    let device = burn::tensor::Device::<Backend>::default();
    let start = Instant::now();

    match TinyLlamaEngine::<Backend>::load_pretrained(2048, &device) {
        Ok(mut engine) => {
            println!("Loaded in {:.2}s\n", start.elapsed().as_secs_f64());
            let gen_start = Instant::now();
            match engine.run(stack, envelope) {
                Ok(result) => {
                    let secs = gen_start.elapsed().as_secs_f64();
                    println!("Output: {}\n", result.text);
                    println!(
                        "Tokens: {} in / {} out ({:.1} tok/s)",
                        result.input_tokens,
                        result.output_tokens,
                        result.output_tokens as f64 / secs,
                    );
                }
                Err(e) => println!("Generation failed: {e:?}"),
            }
        }
        Err(e) => println!("Model load failed: {e}"),
    }
}

#[cfg(all(feature = "llama3", feature = "pretrained", not(feature = "tiny")))]
fn run_inference(
    stack: &converge_llm::PromptStack,
    envelope: &converge_llm::InferenceEnvelope,
) {
    use converge_llm::LlamaEngine;
    use std::time::Instant;

    println!("Loading Llama 3.2 3B...");
    let device = burn::tensor::Device::<Backend>::default();
    let start = Instant::now();

    match LlamaEngine::<Backend>::load_llama3_2_3b(2048, &device) {
        Ok(mut engine) => {
            println!("Loaded in {:.2}s\n", start.elapsed().as_secs_f64());
            let gen_start = Instant::now();
            match engine.run(stack, envelope) {
                Ok(result) => {
                    let secs = gen_start.elapsed().as_secs_f64();
                    println!("Output: {}\n", result.text);
                    println!(
                        "Tokens: {} in / {} out ({:.1} tok/s)",
                        result.input_tokens,
                        result.output_tokens,
                        result.output_tokens as f64 / secs,
                    );
                }
                Err(e) => println!("Generation failed: {e:?}"),
            }
        }
        Err(e) => println!("Model load failed: {e}"),
    }
}

#[cfg(not(any(
    all(feature = "tiny", feature = "pretrained"),
    all(feature = "llama3", feature = "pretrained", not(feature = "tiny"))
)))]
fn run_inference(
    stack: &converge_llm::PromptStack,
    _envelope: &converge_llm::InferenceEnvelope,
) {
    println!("No model backend enabled.");
    println!("Run with: cargo run -p example-local-inference --features \"wgpu,tiny,pretrained\" --release");
    println!("\nPrompt stack validated: version={}", stack.version);
}
