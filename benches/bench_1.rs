use criterion::{criterion_group, criterion_main, Criterion};
use service_robot::{interpreter::Interpreter, parser::DSLParser, scanner::Scanner};
fn bench_1(c: &mut Criterion) {
    c.bench_function("bench_1", |b| {
        b.iter(|| {
            let source = std::fs::read_to_string("scripts/script_simplist.txt").unwrap();
            let mut scanner = Scanner::new(source);
            let commands = scanner.scan().unwrap();
            let mut parser = DSLParser::new();
            parser.parse(commands).unwrap();
            let mut interpreter = Interpreter::new();
            interpreter.interpret(&parser.stages).unwrap();
        });
    });
}

criterion_group!(benches, bench_1);
criterion_main!(benches);
