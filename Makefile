version := 0.1.0
name := git-absorb

# darwin-x86_64: x86_64-apple-darwin
# linux-x86_64: x86_64-unknown-linux-gnu
hardware_name := $(shell uname -m | tr '[:upper:]' '[:lower:]')
os_name := $(shell uname -s | tr '[:upper:]' '[:lower:]')
TARGET_TUPLE := $(hardware_name)-$(os_name)
src_bin := target/release/$(name)
tarball = $(name)-$(version)-$(TARGET_TUPLE).tar.gz

tuple:
	@echo "$(TARGET_TUPLE)"

release: $(tarball)

clean: $(tarball) $(name)
	@rm $^

$(tarball): $(name) LICENSE
	@echo "Compressing"
	@tar -zcvf $@ $^
	@shasum -a 256 $@
	@du -sh $@

$(name): $(src_bin)
	cp $(src_bin) $(name)

$(src_bin):
	cargo build --release

uncompress:
	mkdir -p temp
	tar -zxvf $(tarball) -C temp/

# TODO: Review linux flow
shell:
	docker run --rm -it \
		-v $(PWD):/source \
		-w /source \
		arnau/rust

$(name).linux-x86_64:
	docker run --rm -it \
		-v $(PWD):/source \
		-w /source \
		arnau/rust \
		cargo build --release
