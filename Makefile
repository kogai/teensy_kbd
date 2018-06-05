BIN=teensy_kbd
OUTDIR=target/thumbv7em-none-eabi/release
HEX=$(OUTDIR)/$(BIN).hex
ELF=$(OUTDIR)/$(BIN)

.PHONY: $(ELF) flash clean

all:: $(ELF)

$(ELF):
	cargo xbuild --target=thumbv7em-none-eabi --release

$(HEX): $(ELF)
	arm-none-eabi-objcopy -O ihex $(ELF) $(HEX)

flash: $(HEX)
	teensy-loader-cli -w -mmcu=mk20dx128 $(HEX) -v

clean:
	cargo clean
