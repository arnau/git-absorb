
# i686-unknown-linux-gnu
release-linux:
	cargo build --release
	cp target/release/absorb git-absorb.linux-i386
	rm -rf target/release

# x86_64-apple-darwin
release-macos:
	cargo build --release
	cp target/release/absorb git-absorb.macos-x86_64
	rm -rf target/release
