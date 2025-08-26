## join

Concatenate files to markdown.

### Install

```bash
cargo install --git https://github.com/Pingid/join
```

### Usage

```bash
join [--exclude|-e <GLOB>] [--no-default-excludes|-E] [--version|-V] <FILE> [FILE...]

# or pipe a list of paths (one per line)
printf "src/main.rs\nREADME.md\n" | join
```

### Example

```bash
join ./src/*.rs > lib.md

# exclude paths matching a glob (repeatable)
join --exclude 'target/*' .
join -e '*.log' -e 'node_modules/*' .

# disable built-in defaults and include everything unless excluded explicitly
join --no-default-excludes .
join -E .
```

- **no args**: prints usage
- **errors**: prints message and exits with code 1

### Notes

- Accepts file and folder paths. Folder paths are traversed recursively.
- When stdin is not a TTY (piped), newline-separated paths from stdin are included alongside any CLI args.
- Supports `--exclude <GLOB>` to skip printing or traversing paths that match the glob. Patterns support `*`, `?`, `[...]`, `**` and match against the full path.
- Default excludes (can be disabled with `--no-default-excludes` or `-E`):
  - Dotfiles and dot-directories (e.g., `.git/`, `.env`, `.vscode/`, etc.)
  - Dirs: `target/`, `node_modules/`, `dist/`, `build/`, `out/`, `coverage/`, `vendor/`, `__pycache__/`
  - Lockfiles: `Cargo.lock`, `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`, `bun.lock`, `Pipfile.lock`, `poetry.lock`, `Gemfile.lock`, `composer.lock`, `go.sum`, `npm-shrinkwrap.json`
  - Files: `*.pyc`, `*.pyo`, `*.pyd`, `*.log`, `Thumbs.db`, `Desktop.ini`
  - Add more via repeated `--exclude`/`-e`.
