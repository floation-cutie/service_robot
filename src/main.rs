use service_robot::{error::Error, interpreter::Interpreter, parser::DSLParser, scanner::Scanner};
use std::io::{self, Write};
use std::process::exit;

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

const USAGE: &str = "Usage: cargo run [dsl_file_path]";
const RUNTIME_ERROR: i32 = 70;
const PARSE_ERROR: i32 = 65;
const IO_ERROR: i32 = 74;
const COMMAND_LINE_ERROR: i32 = 64;
const SCAN_ERROR: i32 = 67;
const INPUT_HINT: &str = "Please input the script path you wanna use: ";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut dsl = DSL::new();
    // 解析命令行传递参数 args[1] 为DSL脚本文件路径
    // 通过cargo run [args] 的args参数以args[1]开始
    match &args[..] {
        [_, path] => match dsl.run(path) {
            Ok(_) => (),
            Err(Error::Parse) => exit(PARSE_ERROR),
            Err(Error::Io(e)) => {
                // 格式化输出错误信息
                eprintln!("IoError: {}", e);
                exit(IO_ERROR);
            }
            Err(Error::Scan) => exit(SCAN_ERROR),
            Err(Error::Runtime) => exit(RUNTIME_ERROR),
        },
        [_] => {
            println!("{}", INPUT_HINT);
            let mut input = String::new();
            io::stdout().flush()?;
            io::stdin().read_line(&mut input)?;
            dsl.run(input.trim())?;
        }
        _ => {
            eprintln!("{}", USAGE);
            exit(COMMAND_LINE_ERROR)
        }
    }
    //  dsl.run("dsl.txt")?;
    Ok(())
}
