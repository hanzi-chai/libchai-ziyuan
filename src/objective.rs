use crate::context::{
    字根安排, 字源上下文, 字源决策, 字源决策变化, 最大码长, 进制
};
use crate::encoder::字源编码器;
use chai::{
    objectives::目标函数, 元素, 棱镜, 编码信息, 部分编码信息, 键位分布信息
};
use rustc_hash::FxHashMap;
use serde::Serialize;
use std::{fmt::Display, iter::zip};

const _分级数: usize = 5;
const _分级大小: [usize; _分级数] = [1500, 3000, 4500, 6000, usize::MAX];

#[derive(Debug, Clone, Serialize)]
pub struct 字源指标 {
    pub 字根数: usize,
    pub 一字简码码长: f64,
    pub 一字选重数: u64,
    pub 一字选重率: f64,
    pub 多字选重数: u64,
    pub 多字选重率: f64,
    pub 组合当量: f64,
    pub 按键分布: FxHashMap<char, f64>,
    pub 按键分布偏差: f64,
}

const 键盘布局: [[char; 10]; 4] = [
    ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
    ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';'],
    ['z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/'],
    ['_', '\'', '-', '=', '[', ']', '\\', '`', ' ', ' '],
];

impl Display for 字源指标 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "一字简码码长：{:.4}；一字全码选重数：{}；一字全码选重率：{:.2}%；字根数：{}\n",
            self.一字简码码长,
            self.一字选重数,
            self.一字选重率 * 100.0,
            self.字根数
        )?;
        write!(
            f,
            "多字全码选重数：{}；多字全码选重率：{:.2}%\n",
            self.多字选重数,
            self.多字选重率 * 100.0,
        )?;
        write!(
            f,
            "组合当量：{:.2}%；按键分布偏差：{:.2}%；用指分布：",
            self.组合当量 * 100.0,
            self.按键分布偏差 * 100.0
        )?;
        for 行 in 键盘布局.iter() {
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
    pub 编码结果: Vec<编码信息>,
    pub 编码结果缓冲: Vec<编码信息>,
    pub 拆分序列: Vec<[元素; 4]>,
    pub 拆分序列缓冲: Vec<[元素; 4]>,
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
        let 拆分序列 = vec![<[元素; 4]>::default(); 上下文.固定拆分.len()];
        let 拆分序列缓冲 = 拆分序列.clone();
        let 编码结果: Vec<_> = 上下文
            .固定拆分
            .iter()
            .map(|x| 编码信息 {
                词长: x.词.chars().count(),
                频率: x.频率,
                全码: 部分编码信息::default(),
                简码: 部分编码信息::default(),
            })
            .collect();
        let 编码结果缓冲 = 编码结果.clone();
        Self {
            编码器,
            编码结果,
            编码结果缓冲,
            拆分序列,
            拆分序列缓冲,
            当量信息,
            键位分布信息,
            棱镜: 上下文.棱镜.clone(),
        }
    }
}

impl 目标函数 for 字源目标函数 {
    type 目标值 = 字源指标;
    type 解类型 = 字源决策;

    /// 计算各个部分编码的指标，然后将它们合并成一个指标输出
    fn 计算(
        &mut self, 解: &字源决策, 变化: &Option<字源决策变化>
    ) -> (字源指标, f64) {
        self.编码结果缓冲.clone_from(&self.编码结果);
        self.拆分序列缓冲.clone_from(&self.拆分序列);
        if let Some(变化) = 变化 {
            if 变化.拆分改变 {
                self.编码器.构建元素序列(解, &mut self.拆分序列缓冲);
            }
        } else {
            self.编码器.构建元素序列(解, &mut self.拆分序列缓冲);
        }
        self.编码器
            .动态编码(解, &self.拆分序列缓冲, &mut self.编码结果缓冲);
        let 长度分界点 = [0, 1, 2, 3, 4].map(|x| 进制.pow(x));
        let mut 一字总频率 = 0;
        let mut 多字总频率 = 0;
        let mut 一字总键数 = 0;
        let mut 一字选重数 = 0;
        let mut 一字选重频率 = 0;
        let mut 多字选重数 = 0;
        let mut 多字选重频率 = 0;
        let mut 总组合数 = 0;
        let mut 总组合当量 = 0.0;
        let mut 按键数向量 = vec![0; 进制 as usize];
        let mut 总键数 = 0;
        for (_序号, 编码信息) in self.编码结果缓冲.iter_mut().enumerate() {
            let 预测实际打法 = if 编码信息.词长 == 1 {
                编码信息.简码.原始编码
            } else {
                编码信息.全码.原始编码
            };
            let 编码长度 = 长度分界点.iter().position(|&x| 预测实际打法 < x).unwrap() as u64;
            if 编码信息.词长 == 1 {
                一字总频率 += 编码信息.频率;
                一字总键数 += 编码信息.频率 * 编码长度;
            } else {
                多字总频率 += 编码信息.频率;
            }
            总键数 += 编码信息.频率 * 编码长度;
            if 编码信息.全码.选重标记 {
                if 编码信息.词长 == 1 {
                    一字选重数 += 1;
                    一字选重频率 += 编码信息.频率;
                } else {
                    多字选重数 += 1;
                    多字选重频率 += 编码信息.频率;
                }
            }
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
            .字根
            .iter()
            .filter(|&(_, x)| x != &字根安排::未选取)
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
        let 一字选重率 = 一字选重频率 as f64 / 一字总频率 as f64;
        let 多字选重率 = 多字选重频率 as f64 / 多字总频率 as f64;
        let 组合当量 = 总组合当量 / 总组合数 as f64;
        let 一字简码码长 = 一字总键数 as f64 / 一字总频率 as f64;
        let 指标 = 字源指标 {
            字根数,
            一字简码码长,
            一字选重数,
            一字选重率,
            多字选重数,
            多字选重率,
            组合当量,
            按键分布,
            按键分布偏差,
        };
        let 目标函数值 = 一字选重率
            + 多字选重率
            + 组合当量 * 0.1
            + 按键分布偏差 * 0.01
            + 一字简码码长 * 0.03
            + 字根数 as f64 * 0.0001;

        if 变化.is_none() {
            self.编码结果.clone_from(&self.编码结果缓冲);
            self.拆分序列.clone_from(&self.拆分序列缓冲);
        }
        (指标, 目标函数值)
    }

    fn 接受新解(&mut self) {
        self.编码结果.clone_from(&self.编码结果缓冲);
        self.拆分序列.clone_from(&self.拆分序列缓冲);
    }
}
