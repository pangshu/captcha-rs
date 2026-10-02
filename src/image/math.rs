use super::Driver;
use crate::config::MATH_OPERATORS;
use crate::error::Result;
use crate::util::random_id;
use rand::Rng;

/// 数学算式验证码驱动
pub struct DriverMath;

impl DriverMath {
    pub fn new() -> Self {
        Self
    }

    /// 随机生成数学算式
    fn random_math_expression() -> (String, i32) {
        let mut rng = rand::thread_rng();
        let op = MATH_OPERATORS[rng.gen_range(0..MATH_OPERATORS.len())];

        let (a, b, result) = match op {
            "+" => {
                let a = rng.gen_range(0..50);
                let b = rng.gen_range(0..50);
                (a, b, a + b)
            }
            "-" => {
                let a = rng.gen_range(10..50);
                let b = rng.gen_range(0..=a);
                (a, b, a - b)
            }
            "×" => {
                let a = rng.gen_range(0..10);
                let b = rng.gen_range(0..10);
                (a, b, a * b)
            }
            // MATH_OPERATORS 仅含上述三种运算符；
            // 未来新增运算符时必须在此补充分支，立即 panic 而非静默算错答案
            _ => unreachable!("unknown operator in MATH_OPERATORS: {}", op),
        };

        let expression = format!("{} {} {} = ?", a, op, b);
        (expression, result)
    }
}

impl Default for DriverMath {
    fn default() -> Self {
        Self::new()
    }
}

impl Driver for DriverMath {
    fn generate(&self) -> Result<(String, String, String)> {
        let id = random_id();
        let (content, result) = Self::random_math_expression();
        let answer = result.to_string();
        Ok((id, content, answer))
    }

    fn name(&self) -> &str {
        "DriverMath"
    }
}
