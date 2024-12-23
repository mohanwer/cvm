run_integration_tests:
	cd cvm_server && cargo run & cd cvm_client  && cargo test --tests

build_all:
	cd infinite_hello && cargo build --quiet
	cd cvm_server && cargo build --quiet
	cd cvm_client && cargo build --quiet
