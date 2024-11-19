
parser_test:
    cargo test parser_test -- --test-threads=1 --nocapture

scanner_test:
    cargo test scanner_test -- --test-threads=1 --nocapture
