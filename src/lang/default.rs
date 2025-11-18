use super::*;

pub fn spec() -> LangSpec {
    LangSpec {
        exclude: &[
            // Common dependency/build dirs
            "*/target/*",
            "*/node_modules/*",
            "*/dist/*",
            "*/build/*",
            "*/out/*",
            "*/coverage/*",
            "*/vendor/*",
            "*/__pycache__/*",
            // Lock files
            "*Cargo.lock",
            "*package-lock.json",
            "*yarn.lock",
            "*pnpm-lock.yaml",
            "*bun.lock",
            "*Pipfile.lock",
            "*poetry.lock",
            "*Gemfile.lock",
            "*composer.lock",
            "*go.sum",
            "*npm-shrinkwrap.json",
            // Compiled/binary-ish artifacts and logs
            "*.pyc",
            "*.pyo",
            "*.pyd",
            "*.log",
            // OS/editor cruft
            "*Thumbs.db",
            "*Desktop.ini",
        ],
        sort: SpecSort::InOrder(vec![
            "README.md".to_string(),
            "LICENSE".to_string(),
            "CHANGELOG.md".to_string(),
            "CONTRIBUTING.md".to_string(),
            "CODE_OF_CONDUCT.md".to_string(),
            "SECURITY.md".to_string(),
        ]),
        ..Default::default()
    }
}
