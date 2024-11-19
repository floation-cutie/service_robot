use crate::command::{Command, CommandType};
use crate::error::{error, Error};
use std::collections::HashMap;
use std::fmt;
///
/// 表示转移条件及状态，包括匹配块或输入块
/// - 如果在匹配块中找到匹配项，则转移到下一个阶段
/// - 如果在输入块中成功接收完字符串输入到变量中，则转移到下一个阶段
///
#[derive(Debug, PartialEq)]
pub enum Transition {
    /// 匹配块
    Match(Vec<MatchBlock>),
    /// 输入块
    Input(InputBlock),
}

///
/// 匹配块的组成
/// - pattern: 匹配表达式(可以是正则表达式)
/// - next_stage: 匹配成功后转移的阶段
///
#[derive(Debug, PartialEq)]
pub struct MatchBlock {
    pub pattern: String,
    pub next_stage: String,
}

///
/// 输入块的组成
/// - input_var: 输入变量的名称
/// - next_stage: 无条件转移到的阶段
#[derive(Debug, PartialEq)]
pub struct InputBlock {
    pub input_var: String,
    pub next_stage: String,
}

///
/// 阶段块的组成
/// - stage: 当前阶段
/// - speak: 当前输出
/// - transition: 转移方式（匹配或输入）
///
#[derive(Debug, PartialEq)]
pub struct StageBlock {
    pub stage: String,
    pub speak: String,
    pub transition: Transition,
}

impl StageBlock {
    ///
    /// 生成一个新的StageBlock
    ///
    pub fn new(stage: &str, speak: &str, transition: Transition) -> Self {
        StageBlock {
            stage: stage.to_string(),
            speak: speak.to_string(),
            transition,
        }
    }
}

impl fmt::Display for StageBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Stage: {}\n", self.stage)?;
        write!(f, "  Speak: {}\n", self.speak)?;
        match &self.transition {
            Transition::Match(blocks) => {
                for block in blocks {
                    write!(f, "  Match: {} -> {}\n", block.pattern, block.next_stage)?;
                }
            }
            Transition::Input(block) => {
                write!(f, "  Input: {} -> {}\n", block.input_var, block.next_stage)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
enum Status {
    Init,
    Stage,
    Speak,
    Match,
    MatchNext,
    Input,
    InputNext,
    Default,
}

///
/// DSLParser的结构体定义
/// - stages中的StageBlock实现了PartialEq trait,以实现HashMap的比较
///
pub struct DSLParser {
    pub stages: HashMap<String, StageBlock>,
}

impl DSLParser {
    ///
    /// 生成一个新的DSLParser
    ///
    pub fn new() -> Self {
        DSLParser {
            stages: HashMap::new(),
        }
    }

    fn error(&self, line: i32, what_: &str, message: &str) -> Error {
        error(line, what_, message);
        Error::Parse
    }
    ///
    /// 将命令向量解析为DFA状态迁移表，存储在DSLParser的哈希表中
    /// ## 参数列表
    /// * commands: 命令向量
    /// ## 返回值
    /// * 成功返回Ok，失败返回错误
    ///
    pub fn parse(&mut self, commands: Vec<Command>) -> Result<(), Error> {
        let mut current_stage: Option<String> = None;
        let mut current_speak: Option<String> = None;
        let mut current_transition: Option<Transition> = None;
        let mut current_pattern: Option<String> = None;
        let mut status = Status::Init;

        for command in &commands {
            match &command.ctype {
                CommandType::STAGE(stage) => {
                    if status == Status::Init
                        || status == Status::InputNext
                        || status == Status::MatchNext
                    {
                        status = Status::Stage;
                    } else {
                        // Early return explicitly
                        return Err(self.error(
                            command.line,
                            &format!("STAGE {}", stage),
                            "Unexpected Context",
                        ));
                    }
                    // 如果当前阶段不为空，则保存当前阶段
                    if let Some(stage) = current_stage {
                        if let Some(speak) = current_speak {
                            self.stages.insert(
                                stage.clone(),
                                StageBlock::new(&stage, &speak, current_transition.unwrap()),
                            );
                        }
                    }
                    // 保存新的阶段
                    current_stage = Some(stage.clone());
                    current_speak = None;
                    current_transition = None;
                }
                CommandType::SPEAK(speak) => {
                    if status == Status::Stage {
                        status = Status::Speak;
                    } else {
                        return Err(self.error(
                            command.line,
                            &format!("SPEAK {}", speak),
                            "Unexpected Context",
                        ));
                    }
                    // 保存当前输出
                    current_speak = Some(speak.clone());
                }
                CommandType::MATCH(pattern) => {
                    if status == Status::Speak || status == Status::MatchNext {
                        status = Status::Match;
                    } else {
                        return Err(self.error(
                            command.line,
                            &format!("MATCH {}", pattern),
                            "Unexpected Context",
                        ));
                    }
                    // 保存当前匹配表达式
                    current_pattern = Some(pattern.clone());
                }
                CommandType::DEFAULT => {
                    if status == Status::Speak || status == Status::MatchNext {
                        status = Status::Default;
                    } else {
                        return Err(self.error(command.line, "DEFAULT", "Unexpected Context"));
                    }
                    // 保存当前匹配表达式
                    current_pattern = Some(".*".to_string());
                }
                CommandType::INPUT(input_var) => {
                    if status == Status::Speak {
                        status = Status::Input;
                    } else {
                        return Err(self.error(
                            command.line,
                            &format!("INPUT {}", input_var),
                            "Unexpected Context",
                        ));
                    }
                    // 保存当前输入变量
                    current_pattern = Some(input_var.clone());
                }
                CommandType::NEXT(next_stage) => match status {
                    Status::Match | Status::Default => {
                        status = Status::MatchNext;
                        if let Some(pattern) = &current_pattern {
                            if let Some(transition) = &mut current_transition {
                                match transition {
                                    Transition::Match(blocks) => {
                                        blocks.push(MatchBlock {
                                            pattern: pattern.clone(),
                                            next_stage: next_stage.clone(),
                                        });
                                    }
                                    _ => {}
                                }
                            } else {
                                current_transition = Some(Transition::Match(vec![MatchBlock {
                                    pattern: pattern.clone(),
                                    next_stage: next_stage.clone(),
                                }]));
                            }
                        }
                    }
                    Status::Input => {
                        status = Status::InputNext;
                        if let Some(pattern) = &current_pattern {
                            current_transition = Some(Transition::Input(InputBlock {
                                input_var: pattern.clone(),
                                next_stage: next_stage.clone(),
                            }));
                        }
                    }
                    _ => {
                        return Err(self.error(
                            command.line,
                            &format!("NEXT {}", next_stage),
                            "Unexpected Context",
                        ));
                    }
                },
            }
        }
        // 最后一个阶段保存
        if let Some(stage) = current_stage {
            if let Some(speak) = current_speak {
                self.stages.insert(
                    stage.clone(),
                    StageBlock::new(&stage, &speak, current_transition.unwrap()),
                );
            }
        }

        Ok(())
    }
}

impl fmt::Display for DSLParser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (k, v) in &self.stages {
            write!(f, "Stage: {}\n", k)?;
            write!(f, "  Speak: {}\n", v.speak)?;
            match &v.transition {
                Transition::Match(blocks) => {
                    for block in blocks {
                        write!(f, "  Match: {} -> {}\n", block.pattern, block.next_stage)?;
                    }
                }
                Transition::Input(block) => {
                    write!(f, "  Input: {} -> {}\n", block.input_var, block.next_stage)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;
    use crate::command::CommandType;

    #[test]
    fn test_dsl_parser() {
        let mut parser = DSLParser::new();
        println!();
        let commands = vec![
            Command::new(CommandType::STAGE("stage1".to_string()), 1),
            Command::new(CommandType::SPEAK("speak1".to_string()), 2),
            Command::new(CommandType::MATCH("pattern1".to_string()), 3),
            Command::new(CommandType::NEXT("stage2".to_string()), 4),
            Command::new(CommandType::MATCH("pattern2".to_string()), 5),
            Command::new(CommandType::NEXT("stage3".to_string()), 6),
            Command::new(CommandType::STAGE("stage2".to_string()), 7),
            Command::new(CommandType::SPEAK("speak2".to_string()), 8),
            Command::new(CommandType::MATCH("pattern3".to_string()), 9),
            Command::new(CommandType::NEXT("stage1".to_string()), 10),
            Command::new(CommandType::DEFAULT, 11),
            Command::new(CommandType::NEXT("stage1".to_string()), 12),
            Command::new(CommandType::STAGE("stage3".to_string()), 13),
            Command::new(CommandType::SPEAK("speak3".to_string()), 14),
            Command::new(CommandType::INPUT("input1".to_string()), 15),
            Command::new(CommandType::NEXT("stage1".to_string()), 16),
        ];
        parser.parse(commands).unwrap();
        println!("{}", parser);
        // test if equals to the expected result
        let mut expected = DSLParser::new();
        expected.stages.insert(
            "stage1".to_string(),
            StageBlock::new(
                "stage1",
                "speak1",
                Transition::Match(vec![
                    MatchBlock {
                        pattern: "pattern1".to_string(),
                        next_stage: "stage2".to_string(),
                    },
                    MatchBlock {
                        pattern: "pattern2".to_string(),
                        next_stage: "stage3".to_string(),
                    },
                ]),
            ),
        );
        expected.stages.insert(
            "stage2".to_string(),
            StageBlock::new(
                "stage2",
                "speak2",
                Transition::Match(vec![
                    MatchBlock {
                        pattern: "pattern3".to_string(),
                        next_stage: "stage1".to_string(),
                    },
                    MatchBlock {
                        pattern: ".*".to_string(),
                        next_stage: "stage1".to_string(),
                    },
                ]),
            ),
        );
        expected.stages.insert(
            "stage3".to_string(),
            StageBlock::new(
                "stage3",
                "speak3",
                Transition::Input(InputBlock {
                    input_var: "input1".to_string(),
                    next_stage: "stage1".to_string(),
                }),
            ),
        );
        assert_eq!(parser.stages, expected.stages);
    }

    #[test]
    fn test_dsl_parser_error() {
        let mut parser = DSLParser::new();
        let commands = vec![
            Command::new(CommandType::STAGE("stage1".to_string()), 1),
            Command::new(CommandType::SPEAK("speak1".to_string()), 2),
            Command::new(CommandType::MATCH("pattern1".to_string()), 3),
            Command::new(CommandType::NEXT("stage2".to_string()), 4),
            Command::new(CommandType::MATCH("pattern2".to_string()), 5),
            Command::new(CommandType::NEXT("stage3".to_string()), 6),
            Command::new(CommandType::STAGE("stage2".to_string()), 7),
            Command::new(CommandType::SPEAK("speak2".to_string()), 8),
            Command::new(CommandType::MATCH("pattern3".to_string()), 9),
            Command::new(CommandType::NEXT("stage1".to_string()), 10),
            Command::new(CommandType::STAGE("stage3".to_string()), 11),
            Command::new(CommandType::SPEAK("speak3".to_string()), 12),
            Command::new(CommandType::INPUT("input1".to_string()), 13),
            Command::new(CommandType::NEXT("stage1".to_string()), 14),
            Command::new(CommandType::SPEAK("speak4".to_string()), 15),
        ];
        println!();
        let result = parser.parse(commands);
        let ans = if let Err(Error::Parse) = result {
            true
        } else {
            false
        };
        assert_eq!(ans, true);
    }
}
