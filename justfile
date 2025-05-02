all: build

build:
  cargo build --all-targets --all-features

doc:
  bacon doc

dev-setup:
  # Install the required tools.
  cargo install --locked just bacon

publish:
  cargo publish -p gameson
