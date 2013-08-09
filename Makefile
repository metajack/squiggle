RUSTC ?= rustc

RUST_SRC = $(shell find src -type f -name '*.rs')

squiggle: $(RUST_SRC) src/private.key
	@echo "compile: $@"
	@$(RUSTC) -o $@ src/bin.rs

bench: squiggle-test
	@./squiggle-test --bench

test: squiggle-test
	@./squiggle-test

squiggle-test: $(RUST_SRC) src/private.key
	@echo "compile: $@"
	@$(RUSTC) -o $@ src/bin.rs --test

.PHONY: clean
clean:
	@echo "cleaning"
	@rm -rf squiggle squiggle.dSYM
