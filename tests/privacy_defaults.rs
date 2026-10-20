use std::{fs, path::{Path, PathBuf}, process::Command, time::Duration};

const INSTALLER_PRIVACY_FLAGS: &[&str] =
    &["--privacy", "/privacy", "--Privacy", "/PRIVACY", "--pRiVaCy", "/pRiVaCy"];

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("target")
            .join(format!("privacy-fixture-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}

#[test]
fn privacy_is_available_offline_without_a_data_root() {
    for exe in [env!("CARGO_BIN_EXE_arterm"), env!("CARGO_BIN_EXE_arterm-host")] {
        assert_offline_privacy(exe, "--privacy");
    }
}

#[cfg(feature = "installers")]
#[test]
fn installer_privacy_is_case_insensitive_and_available_without_a_data_root() {
    for exe in [
        env!("CARGO_BIN_EXE_arTerm-Client-Setup"),
        env!("CARGO_BIN_EXE_arTerm-Host-Setup"),
    ] {
        for flag in INSTALLER_PRIVACY_FLAGS {
            assert_offline_privacy(exe, flag);
        }
    }
}

#[test]
#[ignore = "requires explicit ARTERM_PRIVACY_PACKAGE_DIR containing native packaged executables"]
fn packaged_privacy_is_available_offline_without_a_data_root() {
    let directory = PathBuf::from(std::env::var_os("ARTERM_PRIVACY_PACKAGE_DIR")
        .expect("ARTERM_PRIVACY_PACKAGE_DIR is required when selecting the packaged privacy gate"));
    assert!(directory.is_absolute(), "ARTERM_PRIVACY_PACKAGE_DIR must be absolute");
    let directory = directory.canonicalize().expect("packaged privacy directory must exist");
    let executables = [
        ("arterm.exe", false), ("arterm-host.exe", false),
        ("arTerm-Client-Setup.exe", true), ("arTerm-Host-Setup.exe", true),
    ];
    for (name, _) in executables {
        assert!(directory.join(name).is_file(), "missing packaged executable: {name}");
    }
    assert_eq!(fs::read(directory.join("PRIVACY.md")).expect("packaged PRIVACY.md must exist"),
        fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("PRIVACY.md")).unwrap());
    for (name, installer) in executables {
        let flags = if installer { INSTALLER_PRIVACY_FLAGS } else { &["--privacy"] };
        for flag in flags {
            assert_offline_privacy(directory.join(name), flag);
        }
    }
}

fn assert_offline_privacy(exe: impl AsRef<Path>, flag: &str) {
    let exe = exe.as_ref();
    let display = exe.display();
    let fixture = Fixture::new();
    let mut command = Command::new(exe);
    command.arg(flag).env_remove("LOCALAPPDATA")
        .env_remove("VSTERM_REMOTE_HOME").current_dir(&fixture.0);
    for name in ["USERPROFILE", "HOME", "APPDATA", "TEMP", "TMP"] {
        command.env(name, &fixture.0);
    }
    let output = arterm::transport::output_bounded(command, Duration::from_secs(10)).unwrap();
    assert!(output.status.success(), "{display} {flag}: {}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty(), "{display} {flag} unexpectedly wrote stderr");
    assert_eq!(String::from_utf8(output.stdout).unwrap().replace("\r\n", "\n").trim(),
        arterm::deployment::PRIVACY.replace("\r\n", "\n").trim());
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 0);
}

#[test]
fn default_root_selection_and_explicit_override_do_not_migrate_state() {
    for (new, legacy, explicit) in [
        (false, false, false), (true, false, false), (false, true, false),
        (true, true, false), (true, true, true),
    ] {
        let fixture = Fixture::new();
        let current = fixture.0.join("arTerm");
        let old = fixture.0.join("VsTerm");
        let override_root = fixture.0.join("override");
        if new { fs::create_dir(&current).unwrap(); }
        if legacy {
            fs::create_dir(&old).unwrap();
            fs::write(old.join("recovery-sentinel"), b"unchanged").unwrap();
        }
        let mut command = Command::new(env!("CARGO_BIN_EXE_arterm"));
        command.env("LOCALAPPDATA", &fixture.0).env_remove("VSTERM_REMOTE_HOME")
            .args(["add", "local-alias", "--tunnel", "tunnel-name", "--host-path",
                r"C:\Tools\arTerm\Host\arterm-host.exe"]);
        if explicit { command.env("VSTERM_REMOTE_HOME", &override_root); }
        let output = arterm::transport::output_bounded(command, Duration::from_secs(10)).unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let selected = if explicit { &override_root } else if legacy { &old } else { &current };
        assert!(selected.join(r"client\config.json").is_file());
        for other in [&current, &old, &override_root] {
            if other != selected { assert!(!other.join("client").exists()); }
        }
        if legacy { assert_eq!(fs::read(old.join("recovery-sentinel")).unwrap(), b"unchanged"); }
    }
}

#[test]
fn privacy_notice_is_linked_and_in_the_public_artifact_contract() {
    for text in [
        include_str!("../README.md"), include_str!("../QUICKSTART.md"),
        include_str!("../DEVELOPMENT-INSTALL.md"),
    ] {
        assert!(text.contains("](PRIVACY.md)"));
    }
    let ci = include_str!("../.github/workflows/ci.yml");
    assert!(ci.contains("'PRIVACY.md'"));
    assert!(ci.contains(r"dist\windows-${{ matrix.arch }}\PRIVACY.md"));
    assert!(ci.contains("exactly eight files"));
}
