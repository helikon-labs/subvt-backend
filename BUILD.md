<p align="center">
	<img width="400" src="https://raw.githubusercontent.com/helikon-labs/subvt/main/assets/design/logo/subvt_logo_blue.png">
</p>

# Building SubVT

SubVT in testing and production environments is run on [Docker](https://www.docker.com/) using [Docker Compose](https://docs.docker.com/compose/),
however it's also possible to build and run standalone executables.

## Standalone Executables

Make sure you have the latest Rust toolchain and Docker installed. You also need to have `wasm32-unknown-unknown` added to your
Rust toolchain as a target. You can install it with the command `rustup target add wasm32-unknown-unknown`. Then follow the
steps below.

1. `git clone https://github.com/helikon-labs/subvt-backend.git`
2. `cd subvt-backend`
3. `cargo build` for the debug build, or `cargo build --release` for the release build (recommended).

## Docker Containers

Just execute the shell script [_docker/docker_build.sh](./_docker/docker-build.sh) to build Docker container images for
all SubVT executable components. Images will be built and installed into your local Docker repository.