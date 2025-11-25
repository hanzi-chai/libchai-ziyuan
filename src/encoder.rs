use crate::context::{
    动态拆分项, 固定拆分项, 字源上下文, 字源决策, 字源决策变化, 最大码长, 特简字, 特简码, 空格, 进制
};
use chai::{
    encoders::编码器, 元素, 元素映射, 棱镜, 编码信息, 部分编码信息, 错误
};
use rustc_hash::FxHashMap;
use std::iter::zip;

pub struct 字源编码器 {
    pub 固定拆分: Vec<固定拆分项>,
    pub 动态拆分: Vec<动态拆分项>,
    pub 拆分序列: Vec<[元素; 4]>,
    pub _块转数字: FxHashMap<String, usize>,
    pub 数字转块: FxHashMap<usize, String>,
    pub 全码编码空间: Vec<u8>,
    pub 简码编码空间: Vec<u8>,
    pub 棱镜: 棱镜,
    pub 字根首笔: Vec<元素>,
    pub 字根笔画: Vec<(元素, 元素, 元素)>,
    pub 编码结果: Vec<编码信息>,
}

impl 字源编码器 {
    pub fn 新建(上下文: &字源上下文) -> Result<Self, 错误> {
        let 编码空间大小 = 进制.pow(最大码长 as u32) as usize;
        let 全码空间 = vec![u8::default(); 编码空间大小];
        let 拆分序列 = vec![<[元素; 4]>::default(); 上下文.固定拆分.len()];
        let 编码结果: Vec<_> = 上下文
            .固定拆分
            .iter()
            .map(|x| 编码信息 {
                词长: 1,
                频率: x.频率,
                全码: 部分编码信息::default(),
                简码: 部分编码信息::default(),
            })
            .collect();
        Ok(Self {
            动态拆分: 上下文.动态拆分.clone(),
            固定拆分: 上下文.固定拆分.clone(),
            拆分序列,
            _块转数字: 上下文.块转数字.clone(),
            数字转块: 上下文.数字转块.clone(),
            全码编码空间: 全码空间.clone(),
            简码编码空间: 全码空间.clone(),
            棱镜: 上下文.棱镜.clone(),
            字根首笔: 上下文.字根首笔.clone(),
            字根笔画: 上下文.字根笔画.clone(),
            编码结果,
        })
    }

    pub fn 构建元素序列(&mut self, 映射: &Vec<u64>, 决策: &字源决策) {
        let mut 当前拆分索引 = vec![[0; 4]; self.动态拆分.len()];
        for (指针, (_块序号, 拆分方式列表)) in
            zip(&mut 当前拆分索引, self.动态拆分.iter().enumerate())
        {
            let mut 找到 = false;
            for 拆分方式 in 拆分方式列表.iter() {
                if 拆分方式.iter().all(|x| *x == 0 || 映射[*x] != 0) {
                    *指针 = *拆分方式;
                    找到 = true;
                    break;
                }
            }
            if !找到 {
                let 块 = &self.数字转块[&_块序号];
                let 拆分方式 = 拆分方式列表.last().unwrap().map(|x| {
                    if x == 0 {
                        "".to_string()
                    } else {
                        self.棱镜.数字转元素[&x].clone()
                    }
                });
                panic!(
                    "未找到 {块:?} 的映射: {拆分方式:?}\n当前决策为: {:?}",
                    决策.打印(&self.棱镜)
                );
            }
        }
        // 刷新单字元素序列
        for (序列, 固定拆分项) in zip(self.拆分序列.iter_mut(), &self.固定拆分) {
            *序列 = [0; 4];
            let mut index = 0;
            for 块序号 in 固定拆分项.字块 {
                if 块序号 == usize::MAX {
                    break;
                }
                for 元素 in 当前拆分索引[块序号] {
                    if 元素 == 0 {
                        break;
                    }
                    序列[index] = 元素;
                    if index <= 2 {
                        index += 1;
                    }
                }
            }
            if 序列[1] == 0 {
                (序列[1], 序列[2], 序列[3]) = self.字根笔画[序列[0]];
            } else if 序列[2] == 0 {
                序列[2] = self.字根首笔[序列[1]];
            } else if 序列[3] == 0 {
                序列[3] = self.字根首笔[序列[2]];
            }
        }
    }

    pub fn 重置空间(&mut self) {
        self.全码编码空间.iter_mut().for_each(|x| {
            *x = 0;
        });
        self.简码编码空间.iter_mut().for_each(|x| {
            *x = 0;
        });
    }

    #[inline(always)]
    fn 全码规则(元素序列: &[元素; 4], 映射: &元素映射) -> u64 {
        映射[元素序列[0]]
            + (映射[元素序列[1]]) * 进制
            + (映射[元素序列[2]]) * 进制 * 进制
            + (映射[元素序列[3]]) * 进制 * 进制 * 进制
    }

    fn 输出全码(&mut self, _决策: &字源决策, 映射: &元素映射) {
        for (序列, 编码信息) in zip(self.拆分序列.iter(), self.编码结果.iter_mut()) {
            let 全码信息 = &mut 编码信息.全码;
            全码信息.原始编码 = Self::全码规则(序列, &映射);
            全码信息.原始编码候选位置 = self.全码编码空间[全码信息.原始编码 as usize];
            self.全码编码空间[全码信息.原始编码 as usize] += 1;
            全码信息.选重标记 = 全码信息.原始编码候选位置 > 0;
        }
    }

    fn 输出简码(&mut self, 映射: &元素映射) {
        for (序号, 编码信息) in self.编码结果.iter_mut().enumerate() {
            编码信息.简码.原始编码候选位置 = 0;
            编码信息.简码.选重标记 = false;
            let 全码 = 编码信息.全码.原始编码;
            let 拆分 = self.拆分序列[序号];
            // 特简码
            let mut 有特简码 = false;
            for (编码, 字) in zip(特简码.iter(), 特简字.iter()) {
                if self.固定拆分[序号].词 == *字 {
                    let 特简 = self.棱镜.键转数字[&编码];
                    编码信息.简码.原始编码 = 特简;
                    有特简码 = true;
                    break;
                }
            }
            if 有特简码 {
                continue;
            }
            // 一级简码（空格）
            let 一简 = 全码 % 进制 + 空格 * 进制;
            if self.简码编码空间[一简 as usize] == 0 {
                编码信息.简码.原始编码 = 一简;
                self.简码编码空间[一简 as usize] += 1;
                continue;
            }
            // 一级简码（笔画）
            let 笔画 = if 拆分[1] == 0 {
                self.字根首笔[拆分[0]]
            } else {
                self.字根首笔[拆分[1]]
            };
            let 准一简 = 全码 % 进制 + 映射[笔画] * 进制;
            if self.简码编码空间[准一简 as usize] == 0 {
                编码信息.简码.原始编码 = 准一简;
                self.简码编码空间[准一简 as usize] += 1;
                continue;
            }
            // 二级简码
            if 全码 > 进制 * 进制 * 进制 {
                let 二简 = 全码 % (进制 * 进制) + 空格 * 进制 * 进制;
                if self.简码编码空间[二简 as usize] == 0 {
                    编码信息.简码.原始编码 = 二简;
                    self.简码编码空间[二简 as usize] += 1;
                    continue;
                }
            }
            // 无简码
            编码信息.简码.原始编码 = 全码;
            编码信息.简码.原始编码候选位置 = self.简码编码空间[全码 as usize];
            self.简码编码空间[全码 as usize] += 1;
            编码信息.简码.选重标记 = 编码信息.简码.原始编码候选位置 > 0;
        }
    }
}

impl 编码器 for 字源编码器 {
    type 解类型 = 字源决策;
    fn 编码(
        &mut self, 决策: &字源决策, 决策变化: &Option<字源决策变化>, _输出: &mut [编码信息]
    ) {
        let 映射 = 决策.线性化(&self.棱镜);
        if let Some(变化) = 决策变化 {
            if 变化.增加字根.len() > 0 || 变化.减少字根.len() > 0 {
                self.构建元素序列(&映射, 决策);
            }
        } else {
            self.构建元素序列(&映射, 决策);
        };
        self.重置空间();
        self.输出全码(决策, &映射);
        self.输出简码(&映射);
    }
}
