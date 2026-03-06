TARGET=thumbv6m-none-eabi

debug: $(shell find . -name '*.rs') $(shell find . -name 'Cargo.*')
	cargo build -p impl --target $(TARGET)

build: $(shell find . -name '*.rs') $(shell find . -name 'Cargo.*')
	cargo build -p impl --release --target $(TARGET)
	ln -sf ./target/$(TARGET)/release/libimpl.a \
		rust_impl.a
deps:
	rustup target add $(TARGET)
clean:
	cargo clean
	-rm -f rust_impl.a
