use std::path::Path;
use std::sync::LazyLock;

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

const SECRET_FILES: &[&str] = &[
    ".env",
    ".env.*",
    "*.env",
    ".envrc",
    "*.pem",
    "*.key",
    "*.p12",
    "*.pfx",
    "*.jks",
    "*.keystore",
    "*.crt",
    "*.cer",
    "*.der",
    "id_rsa*",
    "id_dsa*",
    "id_ecdsa*",
    "id_ed25519*",
    "*.ppk",
    ".npmrc",
    ".pypirc",
    ".netrc",
    ".git-credentials",
    ".docker/config.json",
    "credentials",
    "credentials.json",
    "*credentials*.json",
    "service-account*.json",
    "*.tfstate",
    "*.tfstate.backup",
    "*.kdbx",
    ".htpasswd",
];

// Anchoring each glob as `**/<glob>` against the whole path catches path-based entries such as
// `.docker/config.json` even when the walk starts inside `.docker/`.
static BUILTIN: LazyLock<GlobSet> = LazyLock::new(|| {
    let mut set = GlobSetBuilder::new();
    for glob in SECRET_FILES {
        set.add(
            GlobBuilder::new(&format!("**/{glob}"))
                .case_insensitive(true)
                .literal_separator(true)
                .build()
                .expect("built-in Exclusion globs are valid"),
        );
    }
    set.build().expect("built-in Exclusion globs are valid")
});

pub fn is_excluded(path: &Path) -> bool {
    BUILTIN.is_match(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_builtin_secret_glob_is_excluded() {
        let secrets = [
            ".env",
            ".env.local",
            "prod.env",
            ".envrc",
            "server.pem",
            "tls.key",
            "cert.p12",
            "cert.pfx",
            "store.jks",
            "release.keystore",
            "ca.crt",
            "ca.cer",
            "ca.der",
            "id_rsa",
            "id_rsa.pub",
            "id_dsa",
            "id_ecdsa",
            "id_ed25519",
            "putty.ppk",
            ".npmrc",
            ".pypirc",
            ".netrc",
            ".git-credentials",
            ".docker/config.json",
            "credentials",
            "credentials.json",
            "gcp-credentials-prod.json",
            "service-account-ci.json",
            "terraform.tfstate",
            "terraform.tfstate.backup",
            "vault.kdbx",
            ".htpasswd",
        ];
        for name in secrets {
            assert!(is_excluded(Path::new(name)), "{name} should be excluded");
            let nested = format!("/home/me/project/sub/{name}");
            assert!(
                is_excluded(Path::new(&nested)),
                "{nested} should be excluded"
            );
        }
    }

    #[test]
    fn matching_ignores_case() {
        for name in [
            ".ENV",
            "Server.PEM",
            "ID_RSA",
            ".Docker/Config.JSON",
            "Credentials.Json",
        ] {
            assert!(is_excluded(Path::new(name)), "{name} should be excluded");
        }
    }

    #[test]
    fn ordinary_source_files_are_not_excluded() {
        for name in [
            "src/env.rs",
            "environment.ts",
            ".env/readme.md",
            "monkey.rs",
            "docker/config.json",
            "config.json",
            "keys.rs",
            "credentials/readme.md",
            "Cargo.toml",
        ] {
            assert!(
                !is_excluded(Path::new(name)),
                "{name} should not be excluded"
            );
        }
    }
}
