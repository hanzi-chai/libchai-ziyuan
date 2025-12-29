use crate::context::{
    一字信息项, 动态拆分项, 多字信息项, 字源上下文, 字源决策, 字源决策变化, 字源方案, 对齐, 方案,
    最大码长, 特简字, 特简码, 空格, 线性化决策, 进制,
};
use chai::{
    encoders::编码器, 元素, 棱镜, 码表项, 编码, 编码信息, 部分编码信息, 错误
};
use rustc_hash::FxHashMap;
use std::iter::zip;

pub struct 字源编码器 {
    pub 一字信息: Vec<一字信息项>,
    pub 一字索引: Vec<usize>,
    pub 多字信息: Vec<多字信息项>,
    pub 多字索引: Vec<usize>,
    pub 多字转一字: Vec<[usize; 4]>,
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
        let 拆分序列 =
            vec![Default::default(); 上下文.一字信息.len() + 上下文.多字信息.len()];
        let mut 编码结果 = vec![];
        for (i, x) in 上下文.一字信息.iter().enumerate() {
            编码结果.push((
                编码信息 {
                    词长: 1,
                    频率: x.频率,
                    全码: 部分编码信息::default(),
                    简码: 部分编码信息::default(),
                },
                i,
                true,
            ));
        }
        for (i, x) in 上下文.多字信息.iter().enumerate() {
            编码结果.push((
                编码信息 {
                    词长: x.词.chars().count(),
                    频率: x.频率,
                    全码: 部分编码信息::default(),
                    简码: 部分编码信息::default(),
                },
                i,
                false,
            ));
        }
        编码结果.sort_by(|a, b| b.0.频率.partial_cmp(&a.0.频率).unwrap());
        let mut 一字索引 = vec![];
        let mut 多字索引 = vec![];
        let mut 汉字转索引 = FxHashMap::default();
        for (索引, (_, 原始索引, 是一字)) in 编码结果.iter().enumerate() {
            if *是一字 {
                一字索引.push((索引, *原始索引));
                汉字转索引.insert(上下文.一字信息[*原始索引].词, 索引);
            } else {
                多字索引.push((索引, *原始索引));
            }
        }
        一字索引.sort_by(|a, b| a.1.cmp(&b.1));
        多字索引.sort_by(|a, b| a.1.cmp(&b.1));
        let mut 多字转一字 = vec![];
        for 多字信息项 in &上下文.多字信息 {
            let 多字: Vec<_> = 多字信息项.词.chars().map(|x| 汉字转索引[&x]).collect();
            let 转换序列 = 对齐(多字, usize::MAX);
            多字转一字.push(转换序列);
        }
        Ok(Self {
            动态拆分: 上下文.动态拆分.clone(),
            一字信息: 上下文.一字信息.clone(),
            一字索引: 一字索引.iter().map(|x| x.0).collect(),
            多字信息: 上下文.多字信息.clone(),
            多字索引: 多字索引.iter().map(|x| x.0).collect(),
            多字转一字,
            拆分序列,
            _块转数字: 上下文.块转数字.clone(),
            数字转块: 上下文.数字转块.clone(),
            全码编码空间: 全码空间.clone(),
            简码编码空间: 全码空间.clone(),
            棱镜: 上下文.棱镜.clone(),
            字根首笔: 上下文.字根首笔.clone(),
            字根笔画: 上下文.字根笔画.clone(),
            编码结果: 编码结果.iter().map(|x| x.0.clone()).collect(),
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
        for (序号, 一字信息项) in zip(&self.一字索引, &self.一字信息) {
            let 序列 = &mut self.拆分序列[*序号];
            *序列 = [0; 4];
            let mut index = 0;
            for 块序号 in 一字信息项.字块 {
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
            if 方案 == 字源方案::前缀 {
                if 序列[1] == 0 {
                    (序列[1], 序列[2], 序列[3]) = self.字根笔画[序列[0]];
                } else if 序列[2] == 0 {
                    序列[2] = self.字根首笔[序列[1]];
                } else if 序列[3] == 0 {
                    序列[3] = self.字根首笔[序列[2]];
                }
            } else {
                let 全拼顺取 = 一字信息项.全拼顺取;
                if 序列[1] == 0 {
                    (序列[1], 序列[2], 序列[3]) = (全拼顺取[0], 全拼顺取[1], 全拼顺取[2]);
                } else if 序列[2] == 0 {
                    (序列[2], 序列[3]) = (全拼顺取[0], 全拼顺取[1]);
                } else if 序列[3] == 0 {
                    序列[3] = 全拼顺取[0];
                }
            }
        }
        // 刷新多字元素序列
        for (序号, 多字转一字) in zip(&self.多字索引, &self.多字转一字) {
            let mut 序列 = [0; 4];
            let [字一, 字二, 字三, 字四] = *多字转一字;
            if 字三 == usize::MAX {
                // 二字词
                序列[0] = self.拆分序列[字一][0];
                序列[1] = self.拆分序列[字一][1];
                序列[2] = self.拆分序列[字二][0];
                序列[3] = self.拆分序列[字二][1];
            } else if 字四 == usize::MAX {
                // 三字词
                序列[0] = self.拆分序列[字一][0];
                序列[1] = self.拆分序列[字二][0];
                序列[2] = self.拆分序列[字三][0];
                序列[3] = self.拆分序列[字三][1];
            } else {
                // 四字词
                序列[0] = self.拆分序列[字一][0];
                序列[1] = self.拆分序列[字二][0];
                序列[2] = self.拆分序列[字三][0];
                序列[3] = self.拆分序列[字四][0];
            }
            self.拆分序列[*序号] = 序列;
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

    pub fn 生成码表(&self) -> Vec<码表项> {
        let mut 码表 = vec![Default::default(); self.编码结果.len()];
        let 编码结果 = &self.编码结果;
        let 转编码 = |code: 编码| self.棱镜.数字转编码(code).iter().collect();
        for (序号, 词) in zip(&self.一字索引, &self.一字信息) {
            let 编码信息 = &编码结果[*序号];
            let 码表项 = 码表项 {
                name: 词.词.to_string(),
                full: 转编码(编码信息.全码.实际编码),
                full_rank: 编码信息.全码.原始编码候选位置,
                short: 转编码(编码信息.简码.实际编码),
                short_rank: 编码信息.简码.原始编码候选位置,
            };
            码表[*序号] = 码表项;
        }
        for (序号, 词) in zip(&self.多字索引, &self.多字信息) {
            let 编码信息 = &编码结果[*序号];
            let 码表项 = 码表项 {
                name: 词.词.to_string(),
                full: 转编码(编码信息.全码.实际编码),
                full_rank: 编码信息.全码.原始编码候选位置,
                short: 转编码(编码信息.简码.实际编码),
                short_rank: 编码信息.简码.原始编码候选位置,
            };
            码表[*序号] = 码表项;
        }
        码表
    }

    #[inline(always)]
    fn 全码规则(元素序列: &[元素; 4], 映射: &线性化决策) -> u64 {
        映射[元素序列[0]]
            + (映射[元素序列[1]]) * 进制
            + (映射[元素序列[2]]) * 进制 * 进制
            + (映射[元素序列[3]]) * 进制 * 进制 * 进制
    }

    fn 补空格(编码: u64) -> u64 {
        if 编码 < 进制 * 进制 {
            编码 + 空格 * 进制 * 进制
        } else if 编码 < 进制 * 进制 * 进制 {
            编码 + 空格 * 进制 * 进制 * 进制
        } else {
            编码
        }
    }

    fn 输出全码(&mut self, _决策: &字源决策, 映射: &线性化决策) {
        for (序列, 编码信息) in zip(self.拆分序列.iter(), self.编码结果.iter_mut()) {
            let 全码信息 = &mut 编码信息.全码;
            全码信息.原始编码 = Self::全码规则(序列, &映射);
            全码信息.实际编码 = Self::补空格(全码信息.原始编码);
            全码信息.原始编码候选位置 = self.全码编码空间[全码信息.原始编码 as usize];
            self.全码编码空间[全码信息.原始编码 as usize] += 1;
            全码信息.选重标记 = 全码信息.原始编码候选位置 > 0;
        }
    }

    fn 输出简码(&mut self, 映射: &线性化决策) {
        for (序号, 编码信息) in self.编码结果.iter_mut().enumerate() {
            编码信息.简码.原始编码候选位置 = 0;
            编码信息.简码.选重标记 = false;
            let 全码 = 编码信息.全码.原始编码;
            let 拆分 = self.拆分序列[序号];
            if 编码信息.词长 == 1 {
                if 方案 == 字源方案::前缀 {
                    // 特简码
                    let mut 有特简码 = false;
                    for (编码, 字) in zip(特简码.iter(), 特简字.iter()) {
                        if self.一字信息[序号].词 == *字 {
                            let 特简 = self.棱镜.键转数字[&编码];
                            编码信息.简码.原始编码 = 特简;
                            有特简码 = true;
                            break;
                        }
                    }
                    if 有特简码 {
                        continue;
                    }
                }
                // 一级简码（空格）
                let 一简 = 全码 % 进制;
                if self.简码编码空间[一简 as usize] == 0 {
                    编码信息.简码.原始编码 = 一简;
                    编码信息.简码.实际编码 = 一简 + 空格 * 进制;
                    self.简码编码空间[一简 as usize] += 1;
                    continue;
                }
                if 方案 == 字源方案::前缀 {
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
                } else {
                    // 二级简码
                    if 全码 > 进制 * 进制 {
                        let 二简 = 全码 % (进制 * 进制);
                        if self.简码编码空间[二简 as usize] == 0 {
                            编码信息.简码.原始编码 = 二简;
                            编码信息.简码.实际编码 = 二简 + 空格 * 进制 * 进制;
                            self.简码编码空间[二简 as usize] += 1;
                            continue;
                        }
                    }
                }
            }
            // 无简码
            编码信息.简码.原始编码 = 全码;
            编码信息.简码.实际编码 = 编码信息.全码.实际编码;
            编码信息.简码.原始编码候选位置 = self.简码编码空间[全码 as usize];
            self.简码编码空间[全码 as usize] += 1;
            编码信息.简码.选重标记 = 编码信息.简码.原始编码候选位置 > 0;
        }
    }
}

impl 编码器 for 字源编码器 {
    type 决策 = 字源决策;
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
