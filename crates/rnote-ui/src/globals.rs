pub(crate) const APP_LICENSE: gtk4::License = gtk4::License::Gpl30;

/// OnePepe: the version shown to the user, with the git commit it was built from when known ("1.2.0 · 8c41d0e"),
/// so a problem can be traced to the exact code. The changes of each version are in VERSIONES.md.
pub(crate) fn onepepe_version() -> String {
    let version = String::from(crate::config::APP_VERSION) + crate::config::APP_VERSION_SUFFIX;
    let commit = env!("ONEPEPE_GIT_COMMIT");
    if commit.is_empty() {
        version
    } else {
        format!("{version} · {commit}")
    }
}
