## join

Concatenate files to stdout.

### Install

```bash
cargo install --git https://github.com/Pingid/join
```

### Usage

```bash
join <FILE> [FILE...]

# or pipe a list of paths (one per line)
printf "a.txt\nb.md\n" | join
```

### Example

```bash
join a.txt b.md
```

- **no args**: prints usage
- **errors**: prints message and exits with code 1

### Notes

- Accepts file and folder paths. Folder paths are traversed recursively.
- When stdin is not a TTY (piped), newline-separated paths from stdin are included alongside any CLI args.
