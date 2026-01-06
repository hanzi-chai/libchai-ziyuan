use crate::context::{
    字源上下文, 字源元素安排, 字源决策, 字源决策变化, 最大码长, 进制
};
use crate::encoder::字源编码器;
use chai::encoders::编码器;
use chai::{objectives::目标函数, 棱镜, 键位分布信息};
use rustc_hash::FxHashMap;
use serde::Serialize;
use std::{fmt::Display, iter::zip};

const _分级数: usize = 5;
const _分级大小: [usize; _分级数] = [1500, 3000, 4500, 6000, usize::MAX];

#[derive(Debug, Clone, Serialize)]
pub struct 字源指标 {
    pub 字根数: usize,
    pub 一字简码码长: f64,
    pub 一字全码选重数: u64,
    pub 一字全码选重率: f64,
    pub 一字简码选重数: u64,
    pub 一字简码选重率: f64,
    pub 多字全码选重数: u64,
    pub 多字全码选重率: f64,
    pub 组合当量: f64,
    pub 按键分布: FxHashMap<char, f64>,
    pub 按键分布偏差: f64,
}

impl Display for 字源指标 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "字根数：{}；一字全码选重数：{}；一字全码选重率：{:.2}%; 一字简码选重数：{}；一字简码选重率：{:.2}%\n",
            self.字根数,
            self.一字全码选重数,
            self.一字全码选重率 * 100.0,
            self.一字简码选重数,
            self.一字简码选重率 * 100.0
        )?;
        write!(
            f,
            "多字全码选重数：{}；多字全码选重率：{:.2}%\n",
            self.多字全码选重数,
            self.多字全码选重率 * 100.0
        )?;
        write!(
            f,
            "一字简码码长：{:.4}；组合当量：{:.2}%；按键分布偏差：{:.2}%；按键分布：",
            self.一字简码码长,
            self.组合当量 * 100.0,
            self.按键分布偏差 * 100.0
        )?;
        for 行 in chai::objectives::metric::键盘布局.iter() {
            if 行.iter().any(|x| self.按键分布.contains_key(x)) {
                f.write_str("\n")?;
                let mut buffer = vec![];
                for 键 in 行 {
                    if let Some(频率) = self.按键分布.get(键) {
                        buffer.push(format!("{} {:5.2}%", 键, 频率 * 100.0));
                    }
                }
                f.write_str(&buffer.join(" | "))?;
            }
        }
        f.write_str("\n")
    }
}

pub struct 字源目标函数 {
    pub 编码器: 字源编码器,
    pub 当量信息: Vec<f64>,
    pub 键位分布信息: 键位分布信息,
    pub 棱镜: 棱镜,
}

impl 字源目标函数 {
    pub fn 新建(上下文: &字源上下文, 编码器: 字源编码器) -> Self {
        let 当量信息 = 上下文
            .棱镜
            .预处理当量信息(&上下文.原始当量信息, 进制.pow(最大码长 as u32) as usize);
        let 键位分布信息 = 上下文.棱镜.预处理键位分布信息(&上下文.原始键位分布信息);
        Self {
            编码器,
            当量信息,
            键位分布信息,
            棱镜: 上下文.棱镜.clone(),
        }
    }
}

impl 目标函数 for 字源目标函数 {
    type 目标值 = 字源指标;
    type 决策 = 字源决策;

    /// 计算各个部分编码的指标，然后将它们合并成一个指标输出
    fn 计算(
        &mut self, 解: &字源决策, 变化: &Option<字源决策变化>
    ) -> (字源指标, f64) {
        self.编码器.编码(解, 变化, &mut vec![]);
        let 长度分界点 = [0, 1, 2, 3, 4].map(|x| 进制.pow(x));
        let mut 一字总频率 = 0;
        let mut 多字总频率 = 0;
        let mut 一字总键数 = 0;
        let mut 一字全码选重数 = 0;
        let mut 一字全码选重频率 = 0;
        let mut 一字简码选重数 = 0;
        let mut 一字简码选重频率 = 0;
        let mut 多字全码选重数 = 0;
        let mut 多字全码选重频率 = 0;
        let mut 总组合数 = 0;
        let mut 总组合当量 = 0.0;
        let mut 按键数向量 = vec![0; 进制 as usize];
        let mut 总键数 = 0;
        for 编码信息 in self.编码器.编码结果.iter() {
            let 预测实际打法 = if 编码信息.词长 == 1 {
                编码信息.简码.实际编码
            } else {
                编码信息.全码.实际编码
            };
            let 编码长度 = if 编码信息.词长 == 1 {
                长度分界点.iter().position(|&x| 预测实际打法 < x).unwrap() as u64
            } else {
                4
            };
            if 编码信息.词长 == 1 {
                一字总频率 += 编码信息.频率;
                一字总键数 += 编码信息.频率 * 编码长度;
            } else {
                多字总频率 += 编码信息.频率;
            }
            if 编码信息.全码.选重标记 {
                if 编码信息.词长 == 1 {
                    一字全码选重数 += 1;
                    一字全码选重频率 += 编码信息.频率;
                } else {
                    多字全码选重数 += 1;
                    多字全码选重频率 += 编码信息.频率;
                }
            }
            if 编码信息.简码.选重标记 {
                if 编码信息.词长 == 1 {
                    一字简码选重数 += 1;
                    一字简码选重频率 += 编码信息.频率;
                }
            }
            总键数 += 编码信息.频率 * 编码长度;
            总组合数 += 编码信息.频率 * (编码长度 - 1);
            总组合当量 += 编码信息.频率 as f64 * self.当量信息[预测实际打法 as usize];
            let mut 剩余编码 = 预测实际打法;
            while 剩余编码 > 0 {
                let 键 = 剩余编码 % 进制;
                按键数向量[键 as usize] += 编码信息.频率;
                剩余编码 /= 进制;
            }
        }

        let 字根数 = 解
            .元素
            .iter()
            .filter(|&x| x != &字源元素安排::未选取)
            .count();
        let 分布: Vec<_> = 按键数向量
            .iter()
            .map(|x| *x as f64 / 总键数 as f64)
            .collect();
        let mut 按键分布偏差 = 0.0;
        for (frequency, loss) in zip(&分布, &self.键位分布信息) {
            let diff = frequency - loss.ideal;
            if diff > 0.0 {
                按键分布偏差 += loss.gt_penalty * diff;
            } else {
                按键分布偏差 -= loss.lt_penalty * diff;
            }
        }
        let mut 按键分布 = FxHashMap::default();
        for (键, 频率) in 按键数向量.iter().enumerate() {
            if let Some(键) = self.棱镜.数字转键.get(&(键 as u64)) {
                按键分布.insert(*键, *频率 as f64 / 总键数 as f64);
            }
        }
        let 一字全码选重率 = 一字全码选重频率 as f64 / 一字总频率 as f64;
        let 一字全码静态选重率 = 一字全码选重数 as f64 / self.编码器.一字信息.len() as f64;
        let 一字简码选重率 = 一字简码选重频率 as f64 / 一字总频率 as f64;
        let 一字简码静态选重率 = 一字简码选重数 as f64 / self.编码器.一字信息.len() as f64;
        let 多字全码选重率 = 多字全码选重频率 as f64 / 多字总频率 as f64;
        let 多字全码静态选重率 = 多字全码选重数 as f64 / self.编码器.多字信息.len() as f64;
        let 组合当量 = 总组合当量 / 总组合数 as f64;
        let 一字简码码长 = 一字总键数 as f64 / 一字总频率 as f64;
        let 指标 = 字源指标 {
            字根数,
            一字简码码长,
            一字全码选重数,
            一字全码选重率,
            一字简码选重数,
            一字简码选重率,
            多字全码选重数,
            多字全码选重率,
            组合当量,
            按键分布,
            按键分布偏差,
        };
        let 目标函数值 = 一字全码选重率 * 0.9 // 不考虑全码
            + 一字全码静态选重率 * 0.2 // 不考虑全码
            + 一字简码选重率 * 0.5 // 固定为 1.0，以此为基准调整其他参数
            + 一字简码静态选重率 * 0.1
            + 多字全码选重率 * 0.3
            + 多字全码静态选重率 * 0.03
            + 组合当量 * 1.0
            // + 按键分布偏差 * 0.01
            // + 一字简码码长 * 0.01
            + 字根数 as f64 * 0.00003;

        (指标, 目标函数值)
    }
}
