use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{env, fs};

use miden_client::account::component::{
    AccountComponentMetadata,
    MIDEN_PACKAGE_EXTENSION,
    basic_fungible_faucet_library,
    basic_wallet_library,
    rpo_falcon_512_library,
};
use miden_client::assembly::Library;
use miden_client::utils::Serializable;
use miden_client::vm::{
    MastArtifact,
    Package,
    PackageExport,
    PackageManifest,
    QualifiedProcedureName,
};

const PACKAGE_DIR: &str = "packages";

fn main() {
    build_component_template(&PathBuf::from("templates/basic-wallet.toml"), basic_wallet_library());

    build_component_template(
        &PathBuf::from("templates/basic-fungible-faucet.toml"),
        basic_fungible_faucet_library(),
    );

    build_component_template(&PathBuf::from("templates/basic-auth.toml"), rpo_falcon_512_library());
}

/// Builds a component template and stores it under `{OUT_DIR}/{PACKAGE_DIR}`.
pub fn build_component_template(metadata_path: &Path, library: Library) {
    let toml_string = fs::read_to_string(metadata_path).expect("failed to read file");

    let template_metadata =
        AccountComponentMetadata::from_toml(&toml_string).expect("faucet toml is well-formed");

    // NOTE: Taken from the miden-compiler's build_package function:
    // Gather all of the procedure metadata for exports of this package
    // https://github.com/0xMiden/compiler/blob/61ee77f57c07c197323728642f8feca972b24217/midenc-compile/src/stages/assemble.rs#L71-L88
    let mut exports: Vec<PackageExport> = Vec::new();
    for module_info in library.module_infos() {
        for (_, proc_info) in module_info.procedures() {
            let name =
                QualifiedProcedureName::new(module_info.path().clone(), proc_info.name.clone());
            let digest = proc_info.digest;
            let signature = proc_info.signature.as_deref().cloned();
            exports.push(PackageExport { name, digest, signature });
        }
    }

    let mast = MastArtifact::Library(Arc::new(library));

    // QUESTION: How to find the dependencies associated with the Library?
    let manifest = PackageManifest::new(exports); //.with_dependencies(dependencies);

    let package = Package {
        name: template_metadata.name().to_string(),
        mast,
        manifest,
        account_component_metadata_bytes: Some(template_metadata.to_bytes()),
    };

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");

    // Write the file
    let packages_out_dir = PathBuf::from(&out_dir).join(PACKAGE_DIR);
    fs::create_dir_all(&packages_out_dir).expect("Failed to create templates directory in OUT_DIR");

    let mut output_filename = metadata_path
        .file_stem()
        .expect("metadata path should have a file stem")
        .to_os_string();
    output_filename.push(format!(".{MIDEN_PACKAGE_EXTENSION}"));

    let output_file = packages_out_dir.join(&output_filename);

    fs::write(&output_file, package.to_bytes()).unwrap_or_else(|e| {
        eprintln!(
            "Failed to write Package {} to file {} in {}. Error: {}",
            package.name,
            output_filename.display(),
            out_dir,
            e
        );
    });
}
