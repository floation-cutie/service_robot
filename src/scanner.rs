use crate::command::{Command, CommandType};
use crate::error::{error, Error};
use regex::Regex;
///
/// scan input strings into commands
///
/// - current 当前解析的位置
///
pub struct Scanner {
    source: String,
    current: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self { source, current: 0 }
    }
    ///
    /// scan input strings into commands
    /// ## 返回值
    /// - 成功返回命令向量，失败返回错误
    pub fn scan(&mut self) -> Result<Vec<Command>, Error> {
        let mut commands: Vec<Command> = Vec::new();
        for line in self.source.lines() {
            self.current += 1;
            if let Some(command) = self.scan_line(line) {
                match command {
                    Ok(cmd) => commands.push(Command::new(cmd, self.current as i32)),
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(commands)
    }

    fn scan_line(&self, line: &str) -> Option<Result<CommandType, Error>> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }
        let re = Regex::new(r"\s+").unwrap(); // 正则表达式
        let mut parts = re.splitn(line, 2);
        let command = parts.next().unwrap();
        let argument = parts.next().unwrap_or("");
        // 同时加上判断argument是否为空的条件
        match command {
            "MATCH" => Some(Ok(CommandType::MATCH(argument.to_string()))),
            "INPUT" => Some(Ok(CommandType::INPUT(argument.to_string()))),
            "SPEAK" => Some(Ok(CommandType::SPEAK(argument.to_string()))),
            "NEXT" => Some(Ok(CommandType::NEXT(argument.to_string()))),
            "STAGE" => Some(Ok(CommandType::STAGE(argument.to_string()))),
            "DEFAULT" => {
                if argument.is_empty() {
                    Some(Ok(CommandType::DEFAULT))
                } else {
                    Some(Err(self.error(line, "Unexpected argument")))
                }
            }
            _ => Some(Err(self.error(line, "Unknown command"))),
        }
    }

    fn error(&self, what_: &str, message: &str) -> Error {
        error(self.current as i32, what_, message);
        Error::Scan
    }
}

#[cfg(test)]
mod scanner_tests {
    use super::*;

    #[test]
    fn test_scan_line_to_command() {
        // notice that Error type can't be compared between each other
        // Internally, there may be dynamic data that cannot be compared
        // (e.g., operating system error codes, descriptions, etc.).
        let placeholder = String::new();
        let scanr = Scanner::new(placeholder);
        let ans = if let Some(Ok(CommandType::MATCH(s))) = scanr.scan_line("MATCH hello") {
            s
        } else {
            String::new()
        };
        assert_eq!(ans, "hello");

        let ans = if let Some(Ok(CommandType::INPUT(s))) = scanr.scan_line("INPUT hello") {
            s
        } else {
            String::new()
        };
        assert_eq!(ans, "hello");

        let ans = if let Some(Ok(CommandType::SPEAK(s))) = scanr.scan_line("SPEAK hello") {
            s
        } else {
            String::new()
        };
        assert_eq!(ans, "hello");

        let ans = if let Some(Ok(CommandType::NEXT(s))) = scanr.scan_line("NEXT hello") {
            s
        } else {
            String::new()
        };
        assert_eq!(ans, "hello");

        let ans = if let Some(Ok(CommandType::DEFAULT)) = scanr.scan_line("DEFAULT") {
            true
        } else {
            false
        };
        assert_eq!(ans, true);
    }

    #[test]
    fn test_scan_line_to_error() {
        let placeholder = String::new();
        let scanr = Scanner::new(placeholder);
        println!();
        let ans = if let Some(Err(Error::Scan)) = scanr.scan_line("DEFAULT shouldn't be here") {
            true
        } else {
            false
        };
        assert_eq!(ans, true);
    }

    #[test]
    fn test_scan_line_to_unknown_command() {
        let placeholder = String::new();
        let scanr = Scanner::new(placeholder);
        println!();
        let ans = if let Some(Err(Error::Scan)) = scanr.scan_line("COMMAND THAT WE DON'T KNOW") {
            true
        } else {
            false
        };
        assert_eq!(ans, true);
    }
    #[test]
    fn test_scan_to_cmds() {
        let source = r#"
            MATCH hello
            INPUT world
            SPEAK hello world
            NEXT hello
            STAGE hello
            DEFAULT
        "#;
        println!();
        let mut scanr = Scanner::new(source.to_string());
        let cmds = scanr.scan().unwrap();
        assert!(cmds.len() == 6);
        assert!(cmds[0].ctype == CommandType::MATCH("hello".to_string()));
        // due to the structure of source string
        assert!(cmds[0].line == 2);
        assert!(cmds[1].ctype == CommandType::INPUT("world".to_string()));
        assert!(cmds[1].line == 3);
        assert!(cmds[2].ctype == CommandType::SPEAK("hello world".to_string()));
        assert!(cmds[2].line == 4);
        assert!(cmds[3].ctype == CommandType::NEXT("hello".to_string()));
        assert!(cmds[3].line == 5);
        assert!(cmds[4].ctype == CommandType::STAGE("hello".to_string()));
        assert!(cmds[4].line == 6);
        assert!(cmds[5].ctype == CommandType::DEFAULT);
        assert!(cmds[5].line == 7);
    }

    #[test]
    fn test_scan_to_error() {
        let source = r#"
            MATCH hello
            INPUT world
            SPEAK hello world
            NEXT hello
            DEFAULT
            UNKNOWN command
        "#;
        println!();
        let mut scanr = Scanner::new(source.to_string());
        let cmds = scanr.scan();
        if cmds.is_err() {
            println!("{}", cmds.as_ref().err().unwrap());
        }
        assert_eq!(cmds.is_err(), true);
    }
}
