use crate::env::GlobalEnvironment;
use crate::error::Error;
use crate::parser::{InputBlock, MatchBlock, StageBlock, Transition};
use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use regex::RegexBuilder;
use std::collections::HashMap;
use std::io::{self, Write};
use std::process::exit;
///
/// DSL解释器
///
pub struct Interpreter {
    /// 全局环境变量
    pub global_env: GlobalEnvironment,
}

impl Interpreter {
    ///
    /// 创建一个新的解释器
    ///
    pub fn new() -> Self {
        Self {
            global_env: GlobalEnvironment::new(),
        }
    }
    ///
    /// 解释DSL
    /// 根据DFA状态迁移表，解释DSL
    ///
    /// # 参数
    /// * stages: DFA状态迁移表
    ///
    /// # 返回值
    /// * 成功返回Ok，失败返回Error
    ///
    pub fn interpret(&mut self, stages: &HashMap<String, StageBlock>) -> Result<(), Error> {
        loop {
            // 当stage get不到时，输出error错误信息
            let stage = stages.get(&self.global_env.stage).ok_or_else(|| {
                self.error(&self.global_env.stage, "Runtime Error", "Stage not found")
            })?;
            // 输出stage.speak,当speak内容中包含变量，且变量未定义时，返回运行时错误
            let speak = self.format_output(&stage.speak)?;
            // println!("DEBUG: the stage is {}", &stage.stage);
            println!("{}", speak);
            io::stdout().flush()?;
            // 判断迁移条件是输入块还是匹配块
            match &stage.transition {
                Transition::Input(input) => {
                    // 输入块
                    self.interpret_input_block(input)?;
                    self.global_env.stage = input.next_stage.clone();
                }
                Transition::Match(match_) => {
                    // 匹配块
                    let match_block = self.interpret_match_blocks(match_)?;
                    self.global_env.stage = match_block.next_stage.clone();
                }
            }
            if self.global_env.stage == "EXIT" {
                break;
            }
        }
        Ok(())
    }
    ///
    /// 解释输入块
    /// 接收用户输入字符串，将之存入全局环境变量
    ///
    /// # 参数
    /// * input: 输入块
    ///
    /// # 返回值
    /// * 成功返回Ok，IO过程失败返回Error
    ///
    fn interpret_input_block(&mut self, input: &InputBlock) -> Result<(), Error> {
        let input_string = self.read_line();
        self.global_env
            .define(input.input_var.clone(), input_string.trim());
        Ok(())
    }

    ///
    /// 解释匹配块
    /// 匹配输入字符串，返回匹配成功的匹配块
    /// 如果没有匹配成功的匹配块，返回运行时错误
    /// 匹配模式支持正则表达式, 且保留匹配关键字EMPTY(没有双引号包裹)
    ///
    /// # 参数
    /// * match_: 匹配块
    ///
    /// # 返回值
    /// * 成功返回匹配成功的匹配块，失败返回运行时错误
    ///
    fn interpret_match_blocks<'a>(
        &self,
        match_: &'a Vec<MatchBlock>,
    ) -> Result<&'a MatchBlock, Error> {
        for match_block in match_ {
            if match_block.pattern == "EMPTY" {
                if match_.len() == 1 {
                    return Ok(match_block);
                } else {
                    return Err(self.error(
                        self.global_env.stage.as_str(),
                        "Runtime Error",
                        "Match pattern 'EMPTY' must be the only pattern",
                    ));
                }
            }
        }
        let input_string = self.read_line();
        let input_string = input_string.trim();
        for match_block in match_ {
            // 去除双引号，且在前面加上^,在后面加上$

            let mut pattern = match_block.pattern.trim().trim_matches('"').to_string();
            // we recommend to use r"pattern" to define a regex pattern
            pattern = format!(r"^{}$", pattern);
            let re = RegexBuilder::new(&pattern)
                .case_insensitive(true)
                .build()
                .unwrap();
            if re.is_match(input_string) {
                return Ok(match_block);
            }
        }
        Err(self.error(
            self.global_env.stage.as_str(),
            "Runtime Error",
            "No match pattern",
        ))
    }

    fn format_output(&self, speak: &str) -> Result<String, Error> {
        // 如果整体被双引号包裹，检查是否需要进一步解析
        if speak.starts_with('"') && speak.ends_with('"') {
            // 如果内部包含 + 号，继续解析
            if speak.contains('+') {
                return self.parse_expression(speak);
            }
            // 否则直接去掉双引号并返回
            return Ok(speak.trim_matches('"').to_string());
        }

        // 处理不带双引号的内容
        self.parse_expression(speak)
    }

    fn parse_expression(&self, speak: &str) -> Result<String, Error> {
        let parts: Vec<&str> = speak.split('+').map(|s| s.trim()).collect();
        let mut result = String::new();

        for part in parts {
            if part.starts_with('"') && part.ends_with('"') {
                // 如果是双引号包裹的字符串，去掉引号
                result.push_str(part.trim_matches('"'));
            } else if let Some(value) = self.global_env.get(part) {
                // 如果是变量，获取变量值
                result.push_str(&value.stringify());
            } else {
                // 如果变量未定义，返回运行时错误
                return Err(self.error(
                    self.global_env.stage.as_str(),
                    "Runtime Error",
                    &format!("Undefined variable '{}'", part),
                ));
            }
        }

        Ok(result)
    }

    ///
    /// 读取用户输入
    /// 支持UTF-8字符集，故支持中文输入
    /// 支持退格键删除，支持Esc键退出,支持Enter键提交输入
    ///
    /// # 返回值
    /// * 返回用户输入的字符串
    ///
    fn read_line(&self) -> String {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode().unwrap(); // 启用原始模式
        stdout.execute(cursor::Hide).unwrap(); // 隐藏光标

        let mut input = String::new(); // 用于存储用户输入的字符串
        loop {
            if let Ok(event) = read() {
                match event {
                    Event::Key(event::KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    }) => {
                        if !input.is_empty() {
                            input.pop(); // 从字符串中删除最后一个字符
                            stdout.execute(cursor::MoveToColumn(0)).unwrap(); // 将光标移动到行首
                            stdout
                                .execute(terminal::Clear(ClearType::CurrentLine))
                                .unwrap(); // 清除当前行内容
                            print!("{}", input); // 重新输出当前的输入字符串
                            stdout.flush().unwrap();
                        }
                    }
                    Event::Key(event::KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    }) => {
                        println!(); // 换行
                        stdout.execute(cursor::MoveToColumn(0)).unwrap(); // 将光标移动到行首
                        stdout
                            .execute(terminal::Clear(ClearType::CurrentLine))
                            .unwrap(); // 清除当前行内容
                        break; // 按Enter键提交输入
                    }
                    Event::Key(event::KeyEvent {
                        code: KeyCode::Esc, ..
                    }) => {
                        input.clear(); // 清空输入
                        stdout.execute(cursor::Show).unwrap(); // 显示光标
                        terminal::disable_raw_mode().unwrap(); // 恢复终端模式
                        exit(0); // 按Esc键退出程序
                    }
                    Event::Key(event::KeyEvent {
                        code: KeyCode::Char(c),
                        ..
                    }) => {
                        input.push(c); // 将字符添加到字符串中
                        print!("{}", c); // 输出字符到屏幕
                        stdout.flush().unwrap();
                    }
                    _ => {}
                }
            }
        }

        stdout.execute(cursor::Show).unwrap(); // 显示光标
        terminal::disable_raw_mode().unwrap(); // 恢复终端模式

        input // 返回最终输入的字符串
    }

    fn error(&self, stage: &str, what_: &str, message: &str) -> Error {
        eprintln!("[stage {}] Error ({}): {}", stage, what_, message);
        Error::Runtime
    }
}

#[cfg(test)]
mod interpreter_tests_user_input {
    use super::*;
    use crate::parser::{InputBlock, MatchBlock, StageBlock, Transition};
    use std::collections::HashMap;

    #[test]
    fn test_interpret_normal_exit() {
        let mut interpreter = Interpreter::new();
        let mut stages = HashMap::new();
        stages.insert(
            "initial".to_string(),
            StageBlock::new(
                "initial",
                "\"Hello, what's your name?\"",
                Transition::Input(InputBlock {
                    input_var: "name".to_string(),
                    next_stage: "next".to_string(),
                }),
            ),
        );
        stages.insert(
            "next".to_string(),
            StageBlock::new(
                "next",
                "\"Hello, \" + name",
                Transition::Match(vec![MatchBlock {
                    pattern: "EMPTY".to_string(),
                    next_stage: "EXIT".to_string(),
                }]),
            ),
        );
        // user input name
        interpreter.interpret(&stages).unwrap();
    }

    #[test]
    fn test_input_block() {
        let mut interpreter = Interpreter::new();
        let input = InputBlock {
            input_var: "name".to_string(),
            next_stage: "next".to_string(),
        };
        interpreter.interpret_input_block(&input).unwrap();
        // user input "world"
        assert_eq!(
            interpreter.global_env.get("name").unwrap().stringify(),
            "world"
        );
    }

    #[test]
    fn test_match_blocks_with_match() {
        let interpreter = Interpreter::new();
        let match_ = vec![MatchBlock {
            pattern: "\"world\"".to_string(),
            next_stage: "EXIT".to_string(),
        }];
        // don't input "world"
        let result = interpreter.interpret_match_blocks(&match_);
        let ans = if let Err(Error::Runtime) = result {
            true
        } else {
            false
        };
        assert_eq!(ans, true);
    }

    #[test]
    fn test_regex_match_blocks_with_match() {
        let interpreter = Interpreter::new();
        let match_ = vec![MatchBlock {
            pattern: "\"[a-z]+\"".to_string(),
            next_stage: "EXIT".to_string(),
        }];
        // input combination of letters(no matter case)
        let result = interpreter.interpret_match_blocks(&match_);
        let ans = if let Ok(match_block) = result {
            match_block.pattern == "\"[a-z]+\""
        } else {
            false
        };
        assert_eq!(ans, true);
    }
}

#[cfg(test)]
mod interpreter_test_subfunction {

    use super::*;
    use crate::parser::MatchBlock;

    #[test]
    fn test_match_blocks_with_more_than_one_empty_trans() {
        let interpreter = Interpreter::new();
        let match_ = vec![
            MatchBlock {
                pattern: "EMPTY".to_string(),
                next_stage: "EXIT".to_string(),
            },
            MatchBlock {
                pattern: "\"world\"".to_string(),
                next_stage: "EXIT".to_string(),
            },
        ];
        let result = interpreter.interpret_match_blocks(&match_);
        let ans = if let Err(Error::Runtime) = result {
            true
        } else {
            false
        };
        assert_eq!(ans, true);
    }

    #[test]
    fn test_match_blocks_with_empty_pattern() {
        let interpreter = Interpreter::new();
        let match_ = vec![MatchBlock {
            pattern: "EMPTY".to_string(),
            next_stage: "EXIT".to_string(),
        }];
        let result = interpreter.interpret_match_blocks(&match_);
        let ans = if let Ok(match_block) = result {
            match_block.pattern == "EMPTY"
        } else {
            false
        };
        assert_eq!(ans, true);
    }
}
