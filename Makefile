RUSTC ?= rustc
RUST_FLAGS ?= -O
RUST_SRC = $(shell find src -type f -name '*.rs')

squiggle: $(RUST_SRC) src/private.key
	@echo "compile: $@"
	@$(RUSTC) $(RUST_FLAGS) -o $@ src/bin.rs

bench: squiggle-test
	@./squiggle-test --bench

test: squiggle-test
	@./squiggle-test

squiggle-test: $(RUST_SRC) src/private.key
	@echo "compile: $@"
	@$(RUSTC) $(RUST_FLAGS) -o $@ src/bin.rs --test

.PHONY: clean
clean:
	@echo "cleaning"
	@rm -rf squiggle squiggle.dSYM squiggle-test squiggle-test.dSYM
