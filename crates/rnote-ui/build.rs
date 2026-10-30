fn main() -> anyhow::Result<()> {
    compile_gresources()?;
    git_commit();

    #[cfg(windows)]
    compile_icon_winres()?;

    Ok(())
}

#[cfg(windows)]
fn compile_icon_winres() -> anyhow::Result<()> {
    use anyhow::Context;

    let mut res = winresource::WindowsResource::new();
    res.set("OriginalFileName", "rnote.exe");
    res.set_icon("./data/icons/rnote.ico");
    res.compile()
        .context("Failed to compile winresource resource")
}

/// OnePepe: the short git commit of the build (empty when git or the repository are not available, e.g. in some
/// Flatpak builds), shown next to the version.
fn git_commit() {
    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    println!("cargo:rustc-env=ONEPEPE_GIT_COMMIT={commit}");
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs/heads");
}

fn compile_gresources() -> anyhow::Result<()> {
    glib_build_tools::compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        "compiled.gresource",
    );
    Ok(())
}
