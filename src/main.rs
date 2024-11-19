use std::io::{self, Write};

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
    fn run(&mut self, path: &str) -> Result<(), Error> {
        let source = std::fs::read_to_string(path)?;
        let mut scanner = Scanner::new(source);
        let commands = scanner.scan()?;
        let mut parser = DSLParser::new();
        parser.parse(commands)?;
        self.interpreter.interpret(&parser.stages)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut dsl = DSL::new();
    dsl.run("dsl.txt")?;
    Ok(())
}
