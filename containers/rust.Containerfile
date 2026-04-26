# Rust dev container: base image + rustup toolchain + rust-analyzer.
#
# Cargo registry is expected to be bind-mounted from the host
# (`~/.cargo/registry`) so crate downloads persist across container
# recreates. Port 3000 is exposed for web dev servers.
#
# Build:  forge dev build rust
# Run:    forge dev up rust

FROM forge-dev-base

USER dev

# Rustup + stable toolchain + rust-analyzer for LSP.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable --component rust-analyzer \
    && . "$HOME/.cargo/env" \
    && rustup component add clippy rustfmt

# Cargo env in profile so every shell picks it up.
RUN echo '. "$HOME/.cargo/env"' >> ~/.zshrc

EXPOSE 3000
