CRATE_NAME:=serde_dokechi

DOC_OPTION:=--no-deps

.PHONY: all
all: build check

.PHONY: build
build: soft-clean
	cargo build

.PHONY: check
check: soft-clean
	cargo fmt -- --check
	cargo test
	cargo clippy -- -D warnings

.PHONY: doc
doc:
	cargo doc $(DOC_OPTION)

.PHONY: doc-open
doc-open:
	cargo doc $(DOC_OPTION) --open

.PHONY: soft-clean
soft-clean:
	cargo clean -p $(CRATE_NAME)

.PHONY: clean
clean:
	cargo clean
