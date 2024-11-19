use std::fmt;

///
/// 对命令类型的枚举定义
///
/// - MATCH(String)
/// - INPUT(String)
/// - SPEAK(String)
/// - NEXT(String)
/// - STAGE(String)
/// - DEFAULT
#[derive(Debug, Clone, PartialEq)]
pub enum CommandType {
    MATCH(String),
    INPUT(String),
    SPEAK(String),
    NEXT(String),
    STAGE(String),
    DEFAULT,
}

///
/// Command类型，包含命令类型和行号
///
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    /// 命令类型
    pub ctype: CommandType,
    /// 行号, 在语法分析过程中适用于定位错误位置
    pub line: i32,
}

impl Command {
    ///
    /// 生成一个新的Command
    ///
    pub fn new(ctype: CommandType, line: i32) -> Self {
        Command { ctype, line }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.ctype {
            CommandType::MATCH(s) => write!(f, "MATCH({})", s),
            CommandType::INPUT(s) => write!(f, "INPUT({})", s),
            CommandType::SPEAK(s) => write!(f, "SPEAK({})", s),
            CommandType::NEXT(s) => write!(f, "NEXT({})", s),
            CommandType::STAGE(s) => write!(f, "STAGE({})", s),
            CommandType::DEFAULT => write!(f, "DEFAULT"),
        }
    }
}
