use std::fmt;
use std::io;

///
/// 输出报错信息
///
/// # 参数列表
/// * line: 报错行数
/// * what_: 报错内容
/// * message: 报错信息
///
/// # 返回值
/// * 无
///
pub fn error(line: i32, what_: &str, message: &str) {
    eprintln!("[line {}] Error ({}): {}", line, what_, message);
}

///
/// 错误的枚举类型
///
#[derive(Debug)]
pub enum Error {
    /// 文件读取错误
    Io(io::Error),
    /// 词法错误
    Scan,
    /// 语法错误
    Parse,
    /// 运行时错误
    Runtime,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(underlying) => write!(f, "IoError {}", underlying),
            Error::Scan => write!(f, "ScanError"),
            Error::Parse => write!(f, "ParseError"),
            Error::Runtime => write!(f, "RuntimeError"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
