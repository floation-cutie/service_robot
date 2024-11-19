///
/// 定义DSL支持的命令
///
pub mod command;
///
/// DSL的环境变量(所有变量均为全局变量)
///
pub mod env;
///
/// 自定义DSL错误
///
pub mod error;
///
/// 定义DSL解释器
///
pub mod interpreter;
///
/// 解析DSL命令向量，得到DSL的DFA状态迁移表
///
pub mod parser;
///
/// 扫描源代码，进行词法分析，得到DSL的命令向量
///
pub mod scanner;
