version := 0.1.0
name := git-absorb
TARGET_TUPLE := $(shell uname -m -s|tr '[:upper:] ' '[:lower:]-')

tuple:
	@echo $(target-tuple)

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


release: $(name)-$(version)-$(TARGET_TUPLE).zip
clean: $(name)-$(version)-$(TARGET_TUPLE).zip
	@rm $^

$(name)-$(version)-$(TARGET_TUPLE).zip: build/$(name).$(TARGET_TUPLE) LICENSE
	@cp build/$(name).$(TARGET_TUPLE) $(name)
	@zip -D $@ $(name) LICENSE
	@shasum -a 256 $@
	@du -sh $@
	@rm $(name)
