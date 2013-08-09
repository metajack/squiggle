RUSTC ?= rustc

RUST_SRC = $(shell find src -type f -name '*.rs')

squiggle: $(RUST_SRC)
	@echo "compile: $@"
	@$(RUSTC) -o $@ src/bin.rs

.PHONY: clean
clean:
	@echo "cleaning"
	@rm -rf squiggle squiggle.dSYM
