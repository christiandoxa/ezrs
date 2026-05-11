# ezrs CLI Workflows

Go pattern mapping:

- `ezrs new myapp` maps to simple binary scaffolding, similar to `go mod init` plus a `cmd` layout.
- `ezrs add command scan` maps to adding a cobra command file.
- `ezrs check` maps to a compact `gofmt`, `go test`, and `go vet` style workflow.

```sh
ezrs new myapp
cd myapp
ezrs add command scan
ezrs run -- hello
ezrs check
ezrs explain --last-error
```

v0.1.0 limitation: `ezrs explain` uses fixed pattern matching. It does not rewrite code.
