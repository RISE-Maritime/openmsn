# openmsn
openMSN aims to provide a reference implementation for integrating geo-distributed (E)MSN-capable simulators using an open-source communication stack.

A proof-of-concept (POC) is available in the `POC` folder.

## Getting Started

### Development Environment (Devcontainer)

This project is designed to work seamlessly in a VS Code devcontainer. The devcontainer includes:

- Rust (latest stable)
- Docker CLI
- bats (for shell-based tests)

To get started:

1. Open the project in VS Code and select "Reopen in Container" if prompted.
2. The devcontainer will automatically set up all required dependencies.
3. You can use the integrated terminal for building, running, and testing the project.

### Manual Prerequisites (if not using devcontainer)

- Rust (latest stable)
- Docker
- bats

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run
```

### Test

To run end-to-end tests:

```bash
bats tests/end2end-docker.bats
```
# openmsn
openMSN aims to provide a reference implementation for integrating geo-distributed (E)MSN-capable simulators using an open-source communication stack.

A proof-of-concept (POC) is available in the `POC` folder.
