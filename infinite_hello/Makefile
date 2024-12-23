add-build-targets:
	rustup install stable
	rustup default stable
	rustup toolchain install stable-x86_64-pc-windows-gnu
	rustup toolchain install stable-x86_64-unknown-linux-gnu
	rustup update

build_release:
	cargo build --release --target x86_64-pc-windows-gnu
	cargo build --release --target x86_64-unknown-linux-gnu

S3_BUCKET=s3://hello-versioned
PKG_VERSION=$(shell cargo pkgid | cut -d "#" -f2)
push_release:
	aws s3 cp ./target/x86_64-pc-windows-gnu/release/infinite_hello.exe $(S3_BUCKET)/windows/infinite_hello_$(PKG_VERSION).exe
	aws s3 cp ./target/x86_64-unknown-linux-gnu/release/infinite_hello $(S3_BUCKET)/linux/infinite_hello_$(PKG_VERSION)