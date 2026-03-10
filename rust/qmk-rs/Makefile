TARGET=thumbv6m-none-eabi

debug: $(shell find . -name '*.rs') $(shell find . -name 'Cargo.*')
	cargo build -p keyboard --target $(TARGET)

build: $(shell find . -name '*.rs') $(shell find . -name 'Cargo.*')
	cargo build -p keyboard --release --target $(TARGET)
	ln -sf ./target/$(TARGET)/release/libkeyboard.a \
		rust_keyboard.a
deps:
	rustup target add $(TARGET)
clean:
	cargo clean
	-rm -f rust_keyboard.a
