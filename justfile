test: parser_test scanner_test env_test interpreter_subfunc_test integration_test

parser_test:
    cargo test parser_test -- --test-threads=1 --nocapture

scanner_test:
    cargo test scanner_test -- --test-threads=1 --nocapture

env_test:
    cargo test env_test -- --test-threads=1 --nocapture

interpreter_subfunc_test:
    cargo test interpreter_test_subfunction -- --test-threads=1 --nocapture

integration_test:
    cargo test --test integration_test -- --test-threads=1 