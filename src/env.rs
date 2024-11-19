use std::collections::HashMap;
use std::fmt;

///
/// 定义DSL支持的数据类型
///
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// 数值
    Number(f64),
    /// 字符串
    String(String),
}

impl Value {
    ///
    /// 将数据类型转换为字符串
    ///
    /// # 返回值
    /// * 转换后的字符串
    ///
    pub fn stringify(&self) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
        }
    }
}

///
/// 定义全局环境变量
///
pub struct GlobalEnvironment {
    /// 全局变量
    pub values: HashMap<String, Value>,
    /// 当前阶段
    pub stage: String,
}

impl GlobalEnvironment {
    ///
    /// 创建一个新的全局环境变量
    /// 默认的阶段为"initial"
    ///
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            stage: "initial".to_string(),
        }
    }
    ///
    /// 定义一个全局变量
    ///
    /// # 参数
    /// * name: 变量名
    /// * value: 变量值
    ///
    /// # 返回值
    /// * 无
    ///
    pub fn define(&mut self, name: String, value: &str) {
        self.values
            .insert(name, self.string_convert_to_value(value));
    }
    ///
    /// 获取一个全局变量
    ///
    /// # 参数
    /// * name: 变量名
    ///
    /// # 返回值
    /// * 成功返回Some(变量值)，失败返回None
    ///
    pub fn get(&self, name: &str) -> Option<Value> {
        self.values.get(name).cloned()
    }

    ///
    /// 将字符串转换为数据类型
    ///
    /// # 参数
    /// * s: 字符串
    ///
    /// # 返回值
    /// * 转换后的数据类型
    ///
    fn string_convert_to_value(&self, s: &str) -> Value {
        if let Ok(number) = s.parse::<f64>() {
            Value::Number(number)
        } else {
            Value::String(s.to_string())
        }
    }
}

impl fmt::Display for GlobalEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "values: {:?}", self.values)
    }
}

#[cfg(test)]
mod env_tests {
    use super::*;

    #[test]
    fn test_get_values() {
        let mut env = GlobalEnvironment::new();
        env.define("a".to_string(), "1");
        env.define("b".to_string(), "hello");
        assert_eq!(env.get("a"), Some(Value::Number(1.0)));
        assert_eq!(env.get("b"), Some(Value::String("hello".to_string())));
    }

    #[test]
    fn test_get_values_not_exist() {
        let env = GlobalEnvironment::new();
        assert_eq!(env.get("a"), None);
    }

    #[test]
    fn test_get_values_override() {
        let mut env = GlobalEnvironment::new();
        env.define("a".to_string(), "1");
        env.define("a".to_string(), "hello");
        assert_eq!(env.get("a"), Some(Value::String("hello".to_string())));
    }

    #[test]
    fn test_initial_stage() {
        let env = GlobalEnvironment::new();
        assert_eq!(env.stage, "initial");
    }
}
