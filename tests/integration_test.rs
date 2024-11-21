use service_robot::{error::Error, interpreter::Interpreter, parser::DSLParser, scanner::Scanner};

struct DSL {
    interpreter: Interpreter,
}

impl DSL {
    fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    ///
    /// 运行DSL
    /// 根据DSL脚本文件路径，解释DSL
    /// # 参数
    /// * path: DSL脚本文件路径
    ///
    /// # 返回值
    /// * 成功返回Ok，失败返回Error
    ///
    fn run(&mut self, path: &str) -> Result<(), Error> {
        let source = std::fs::read_to_string(path)?;
        let mut scanner = Scanner::new(source);
        let commands = scanner.scan()?;
        let mut parser = DSLParser::new();
        parser.parse(commands)?;
        self.interpreter.interpret(&parser.stages)
    }
}

#[test]
fn test_run() {
    let mut dsl = DSL::new();
    let path = "scripts/script_simplist.txt";
    assert!(dsl.run(path).is_ok());
}

#[test]
fn test_run_error() {
    let mut dsl = DSL::new();
    let path = "scripts/script_unknown_stage.txt";
    if let Err(Error::Runtime) = dsl.run(path) {
        assert!(true);
    } else {
        assert!(false);
    }
}

#[test]
fn test_parse_error() {
    let mut dsl = DSL::new();
    let path = "scripts/script_incomplete_block.txt";
    if let Err(Error::Parse) = dsl.run(path) {
        assert!(true);
    } else {
        assert!(false);
    }
}

#[test]
fn test_scan_error() {
    let mut dsl = DSL::new();
    let path = "scripts/script_nonexist_grammar.txt";
    if let Err(Error::Scan) = dsl.run(path) {
        assert!(true);
    } else {
        assert!(false);
    }
}
