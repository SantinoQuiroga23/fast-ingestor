.PHONY: build test clean benchmark publish

build:
	maturin develop --release

build-debug:
	maturin develop

test:
	cargo test
	pytest tests/ -v

clean:
	cargo clean
	rm -rf target/
	rm -rf *.egg-info
	find . -type d -name __pycache__ -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete

benchmark:
	python benchmarks/load_test_single_cpu.py --requests 10000

format:
	cargo fmt
	black python/ tests/ examples/

lint:
	cargo clippy -- -D warnings
	ruff check python/ tests/ examples/