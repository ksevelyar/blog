use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::error::{AppError, Result};

pub struct WasmComponent {
    pub javascript_bindings_path: PathBuf,
    pub wasm_binary_path: PathBuf,
}

pub fn build_circle_component() -> Result<WasmComponent> {
    let project_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let component_manifest_path = project_directory.join("components/circle/Cargo.toml");
    let target_directory = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| project_directory.join("target"));
    let component_wasm_path =
        target_directory.join("wasm32-unknown-unknown/release/circle_component.wasm");
    let binding_output_directory = target_directory.join("generated/wasm/circle");

    run_command(
        Command::new("cargo")
            .arg("build")
            .arg("--locked")
            .arg("--manifest-path")
            .arg(&component_manifest_path)
            .args(["--target", "wasm32-unknown-unknown", "--release"]),
        "failed to compile the circle component for wasm32-unknown-unknown",
    )?;

    if binding_output_directory.exists() {
        fs::remove_dir_all(&binding_output_directory).map_err(|error| {
            AppError::Component(format!(
                "failed to clear generated bindings at {}: {error}",
                binding_output_directory.display()
            ))
        })?;
    }
    fs::create_dir_all(&binding_output_directory).map_err(|error| {
        AppError::Component(format!(
            "failed to create generated bindings directory {}: {error}",
            binding_output_directory.display()
        ))
    })?;

    run_command(
        Command::new("wasm-bindgen")
            .arg("--target")
            .arg("web")
            .arg("--out-dir")
            .arg(&binding_output_directory)
            .arg("--out-name")
            .arg("circle")
            .arg(&component_wasm_path),
        "failed to generate browser JavaScript bindings for the circle component",
    )?;

    let wasm_component = WasmComponent {
        javascript_bindings_path: binding_output_directory.join("circle.js"),
        wasm_binary_path: binding_output_directory.join("circle_bg.wasm"),
    };
    for artifact_path in [
        &wasm_component.javascript_bindings_path,
        &wasm_component.wasm_binary_path,
    ] {
        if !artifact_path.is_file() {
            return Err(AppError::Component(format!(
                "expected generated Wasm artifact was not found: {}",
                artifact_path.display()
            )));
        }
    }

    Ok(wasm_component)
}

fn run_command(command: &mut Command, failure_message: &str) -> Result<()> {
    let command_status = command.status().map_err(|error| {
        AppError::Component(format!(
            "{failure_message}: command could not be started: {error}"
        ))
    })?;
    if !command_status.success() {
        return Err(AppError::Component(format!(
            "{failure_message}: command exited with {command_status}"
        )));
    }
    Ok(())
}
