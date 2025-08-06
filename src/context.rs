use chai::{
    config::{Mapped, 配置},
    contexts::上下文,
    interfaces::默认输入,
    objectives::metric::指法标记,
    optimizers::解特征,
    元素, 元素映射, 原始当量信息, 原始键位分布信息, 棱镜, 码表项, 编码, 编码信息, 错误,
};
use chrono::Local;
use core::panic;
use indexmap::IndexMap;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_yaml::{from_str, to_string};
use std::{
    cmp::Reverse,
    fs::{File, read_to_string},
    io::Write,
    path::PathBuf,
};

pub const 字母表: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];
pub const 最大码长: u64 = 4;
pub const 进制: u64 = 28;
pub const 空格: u64 = 27;

#[derive(Clone)]
pub struct 字源上下文 {
    pub 配置: 配置,
    pub 棱镜: 棱镜,
    pub 初始决策: 字源决策,
    pub 决策空间: 字源决策空间,
    pub 原始键位分布信息: 原始键位分布信息,
    pub 原始当量信息: 原始当量信息,
    pub 固定拆分: Vec<固定拆分项>,
    pub 动态拆分: Vec<动态拆分项>,
    pub 块转数字: FxHashMap<String, usize>,
    pub 数字转块: FxHashMap<usize, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 字源决策 {
    pub 字根: IndexMap<String, 字根安排>,
}

impl 字源决策 {
    pub fn 线性化(&self, 棱镜: &棱镜) -> 元素映射 {
        let mut 映射 = vec![0; 棱镜.数字转元素.len() + 1];
        for 键 in 0..进制 {
            映射[键 as usize] = 键;
        }
        for (元素, 安排) in &self.字根 {
            let 索引 = 棱镜.元素转数字[元素];
            match 安排 {
                字根安排::未选取 => {}
                字根安排::乱序 { 键位 } => {
                    映射[索引] = 棱镜.键转数字[键位];
                }
                字根安排::键位 { 键位 } => {
                    映射[索引] = 棱镜.键转数字[键位];
                }
                字根安排::归并 { 字根 } => {
                    let 字根索引 = 棱镜.元素转数字[字根];
                    映射[索引] = 映射[字根索引];
                }
            }
        }
        映射
    }
}
#[derive(Debug, Clone)]
pub struct 字源决策空间 {
    pub 字根: IndexMap<String, Vec<字根安排>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "类型", rename_all = "snake_case")]
pub enum 字根安排 {
    未选取,
    键位 { 键位: char },
    乱序 { 键位: char },
    归并 { 字根: String },
}

#[derive(Debug, Clone)]
pub struct 字源决策变化 {
    pub 拆分改变: bool,
}

impl 字源决策变化 {
    pub fn 新建() -> Self {
        字源决策变化 {
            拆分改变: false
        }
    }
}

impl 解特征 for 字源决策 {
    type 变化 = 字源决策变化;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct 规则输入 {
    pub 元素: String,
    pub 规则: Vec<字根安排>,
    pub 允许乱序: Option<bool>,
}

impl 上下文 for 字源上下文 {
    type 解类型 = 字源决策;

    fn 序列化(&self, 解: &字源决策) -> String {
        let mut 新配置 = self.配置.clone();
        新配置.info.as_mut().unwrap().version =
            Some(format!("{}", Local::now().format("%Y-%m-%d+%H:%M:%S")));
        let 映射 = 解.线性化(&self.棱镜);
        let mut mapping = IndexMap::new();
        let 全部元素: Vec<_> = 解.字根.keys().cloned().collect();
        for 元素 in &全部元素 {
            let 索引 = self.棱镜.元素转数字[元素];
            let 键 = 映射[索引];
            if 键 == 0 {
                continue;
            }
            let 编码 = self.棱镜.数字转键[&键].to_string();
            let 新键位 = Mapped::Basic(编码);
            mapping.insert(元素.clone(), 新键位);
        }
        新配置.form.mapping = mapping;
        to_string(&新配置).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 分析结果 {
    pub 重码项: Vec<(String, (Vec<String>, u64))>,
    pub 差指法: Vec<(String, String)>,
}

#[derive(Deserialize)]
struct 原始固定拆分项 {
    汉字: String,
    读音: Vec<带频读音>,
    拆分: Vec<String>,
}

type 原始固定拆分 = Vec<原始固定拆分项>;
type 原始动态拆分 = FxHashMap<String, Vec<Vec<String>>>;
type 原始词语读音频率 = Vec<词语带频读音>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct 词语带频读音 {
    词语: String,
    拼音: Vec<String>,
    频率: u64,
}

#[derive(Deserialize)]
struct 带频读音 {
    拼音: String,
    频率: u64,
}

#[derive(Deserialize)]
struct 拆分输入 {
    固定拆分: 原始固定拆分,
    动态拆分: 原始动态拆分,
    词语读音频率: 原始词语读音频率,
}

pub type 块 = usize;
pub type 动态拆分项 = Vec<[元素; 4]>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 固定拆分项 {
    pub 词: String,
    pub 词长: usize,
    pub 频率: u64,
    pub 拼音: (usize, usize),
    pub 字块: [块; 4],
}

impl 字源上下文 {
    pub fn 新建(输入: 默认输入) -> Result<Self, 错误> {
        let 规则列表: Vec<规则输入> = from_str(&read_to_string("rules.yaml")?).unwrap();
        let mut 决策空间 = 字源决策空间 {
            字根: IndexMap::default(),
        };
        let mut 初始决策 = 字源决策 {
            字根: IndexMap::default(),
        };
        let 布局 = 输入.配置.form.clone();
        let 映射 = 布局.mapping;
        let 可选映射 = 布局.optional.unwrap();
        let mut 元素转数字 = FxHashMap::default();
        let mut 数字转元素 = FxHashMap::default();
        let mut 键转数字 = FxHashMap::default();
        let mut 数字转键 = FxHashMap::default();
        let mut 数字 = 0;
        for c in 字母表 {
            数字 += 1;
            元素转数字.insert(c.to_string(), 数字);
            数字转元素.insert(数字, c.to_string());
            键转数字.insert(c, 数字 as u64);
            数字转键.insert(数字 as u64, c);
        }
        数字 += 1; // 空格
        assert!(数字 == 空格 as usize);
        元素转数字.insert("_".to_string(), 数字 as usize);
        数字转元素.insert(数字 as usize, "_".to_string());
        键转数字.insert('_', 数字 as u64);
        数字转键.insert(数字 as u64, '_');
        let 投影 = |编码: &Mapped| {
            let Mapped::Basic(s) = 编码 else {
                panic!("编码必须是基本类型");
            };
            s.to_string()
        };
        for 规则输入 {
            元素,
            规则,
            允许乱序,
        } in &规则列表
        {
            if 元素.contains("母-") {
                continue; // 跳过声母
            }
            let 允许乱序 = 允许乱序.unwrap_or(false);
            let 编码 = &映射.get(元素).unwrap_or_else(|| &可选映射[元素]);
            let 元素 = 元素.clone();
            let 编码 = 投影(编码);
            数字 += 1;
            元素转数字.insert(元素.clone(), 数字);
            数字转元素.insert(数字, 元素.clone());
            let mut 规则 = 规则.to_vec();
            if 可选映射.contains_key(&元素) {
                规则.push(字根安排::未选取);
            }
            if 允许乱序 {
                for 键位 in 字母表 {
                    规则.push(字根安排::乱序 { 键位 });
                }
            }
            let mut 匹配 = false;
            for 安排 in &规则 {
                匹配 = match 安排 {
                    字根安排::键位 { 键位 } => 编码 == 键位.to_string(),
                    字根安排::归并 { 字根 } => {
                        映射.contains_key(字根) && 编码 == 投影(&映射[字根])
                    }
                    字根安排::乱序 { 键位 } => 编码 == 键位.to_string(),
                    字根安排::未选取 => 编码 == "a",
                };
                if 匹配 {
                    初始决策.字根.insert(元素.clone(), 安排.clone());
                    break;
                }
            }
            if !匹配 {
                panic!("字根 {元素:?} 的编码 {编码:?} 在规则中没有匹配到");
            }
            决策空间.字根.insert(元素.clone(), 规则);
        }

        let mut 所有乱序键位: Vec<_> = 初始决策
            .字根
            .iter()
            .filter_map(|(_, 安排)| {
                if let 字根安排::乱序 { 键位, .. } = 安排 {
                    Some(*键位)
                } else {
                    None
                }
            })
            .collect();
        所有乱序键位.sort();
        println!("所有乱序键位: {:?}", 所有乱序键位);
        assert!(所有乱序键位.len() == 4);

        let 棱镜 = 棱镜 {
            键转数字,
            数字转键,
            元素转数字,
            数字转元素,
            进制,
        };
        let (固定拆分, 动态拆分, 块转数字, 数字转块) = Self::解析动态拆分(&棱镜, &决策空间);

        Ok(Self {
            配置: 输入.配置,
            棱镜,
            初始决策,
            决策空间,
            原始键位分布信息: 输入.原始键位分布信息,
            原始当量信息: 输入.原始当量信息,
            固定拆分,
            动态拆分,
            块转数字,
            数字转块,
        })
    }

    fn 对齐(列表: Vec<元素>, 默认值: 元素) -> [元素; 4] {
        [0, 1, 2, 3].map(|i| {
            if i == 3 && 列表.len() > 3 {
                列表[列表.len() - 1]
            } else if i < 列表.len() {
                列表[i]
            } else {
                默认值
            }
        })
    }

    pub fn 解析动态拆分(
        棱镜: &棱镜,
        决策空间: &字源决策空间,
    ) -> (
        Vec<固定拆分项>,
        Vec<动态拆分项>,
        FxHashMap<String, usize>,
        FxHashMap<usize, String>,
    ) {
        let 拆分输入: 拆分输入 =
            from_str(&read_to_string("dynamic_analysis.yaml").unwrap()).unwrap();
        let mut 动态拆分 = vec![];
        let mut 块转数字 = FxHashMap::default();
        let mut 数字转块 = FxHashMap::default();
        for (块, 原始拆分方式列表) in 拆分输入.动态拆分 {
            let 块序号 = 动态拆分.len();
            块转数字.insert(块.clone(), 块序号);
            数字转块.insert(块序号, 块.clone());
            let mut 拆分方式列表 = vec![];
            for 原始拆分方式 in &原始拆分方式列表 {
                for 拆分方式 in 原始拆分方式 {
                    assert!(
                        棱镜.元素转数字.contains_key(拆分方式),
                        "元素 {} 不在棱镜中",
                        拆分方式
                    );
                }
                let 拆分方式 = Self::对齐(
                    原始拆分方式
                        .iter()
                        .map(|字根| 棱镜.元素转数字[字根])
                        .collect(),
                    0_usize,
                );
                拆分方式列表.push(拆分方式);
            }
            // 检查原始拆分方式列表的最后一项都是必选字根
            let 最后一项 = 原始拆分方式列表.last().unwrap();
            if !最后一项
                .iter()
                .all(|x| !决策空间.字根[x].contains(&字根安排::未选取))
            {
                panic!("动态拆分方式列表的最后一项必须都是必选字根, {块:?}, {原始拆分方式列表:?}");
            }
            动态拆分.push(拆分方式列表);
        }
        let mut 固定拆分 = vec![];
        let mut 所有合法汉字 = FxHashMap::default();
        for 词 in &拆分输入.固定拆分 {
            let 字块 = Self::对齐(词.拆分.iter().map(|块| 块转数字[块]).collect(), usize::MAX);
            let 频率 = 词.读音.iter().map(|x| x.频率).sum();
            let 最高频读音 = &词.读音.iter().max_by_key(|&x| x.频率).unwrap().拼音;
            let 拼音首 = 最高频读音.chars().next().unwrap();
            let 拼音末 = 最高频读音
                .chars()
                .nth(最高频读音.chars().count() - 2)
                .unwrap();
            let 拼音首 = 棱镜.键转数字[&拼音首];
            let 拼音末 = 棱镜.键转数字[&拼音末];
            所有合法汉字.insert(词.汉字.chars().next().unwrap(), 0);
            固定拆分.push(固定拆分项 {
                词: 词.汉字.clone(),
                词长: 1,
                频率,
                拼音: (拼音首 as usize, 拼音末 as usize),
                字块,
            });
        }
        for 词 in &拆分输入.词语读音频率 {
            if !词.词语.chars().all(|c| 所有合法汉字.contains_key(&c)) {
                println!("词语 {} 中包含不合法的汉字", 词.词语);
                continue;
            }
            let 词长 = 词.词语.chars().count();
            let 拼音一 = 棱镜.键转数字[&词.拼音[词长 - 2].chars().next().unwrap()];
            let 拼音二 = 棱镜.键转数字[&词.拼音[词长 - 1].chars().next().unwrap()];
            固定拆分.push(固定拆分项 {
                词: 词.词语.clone(),
                词长,
                频率: 词.频率,
                拼音: (拼音一 as usize, 拼音二 as usize),
                字块: [0, 0, 0, 0],
            });
        }
        固定拆分.sort_by_key(|x| Reverse(x.频率));
        // 刷新汉字的索引
        for (索引, 拆分项) in 固定拆分.iter().enumerate() {
            if 拆分项.词长 == 1 {
                所有合法汉字.insert(拆分项.词.chars().next().unwrap(), 索引);
            }
        }
        for 拆分项 in 固定拆分.iter_mut() {
            if 拆分项.词长 > 1 {
                let 所有索引: Vec<_> = 拆分项.词.chars().map(|c| 所有合法汉字[&c]).collect();
                拆分项.字块 = Self::对齐(所有索引, usize::MAX);
            }
        }
        (固定拆分, 动态拆分, 块转数字, 数字转块)
    }

    pub fn 生成码表(&self, 编码结果: &[编码信息]) -> Vec<码表项> {
        let mut 码表 = Vec::new();
        let 转编码 = |code: 编码| self.棱镜.数字转编码(code).iter().collect();
        for (序号, 可编码对象) in self.固定拆分.iter().enumerate() {
            let 码表项 = 码表项 {
                name: 可编码对象.词.clone(),
                full: 转编码(编码结果[序号].全码.原始编码),
                full_rank: 编码结果[序号].全码.原始编码候选位置,
                short: 转编码(编码结果[序号].简码.原始编码),
                short_rank: 编码结果[序号].简码.原始编码候选位置,
            };
            码表.push(码表项);
        }
        码表
    }

    // 分析前 3000 字中全码重码和简码差指法的情况
    pub fn 分析码表(&self, 码表: &[码表项], 路径: &PathBuf) {
        let 指法标记 = 指法标记::new();
        let mut 文件 = File::create(路径).unwrap();
        let mut 翻转码表 = FxHashMap::default();
        for (序号, 码表项) in 码表[..3000].iter().enumerate() {
            let 记录 = 翻转码表
                .entry(码表项.full.clone())
                .or_insert_with(|| (vec![], 0));
            记录.0.push(码表项.name.clone());
            if 记录.0.len() == 2 {
                记录.1 = self.固定拆分[序号].频率;
            }
            for 键索引 in 0..(码表项.short.len() - 1) {
                let 组合 = (
                    码表项.short.chars().nth(键索引).unwrap(),
                    码表项.short.chars().nth(键索引 + 1).unwrap(),
                );
                if 指法标记.同指大跨排.contains(&组合) || 指法标记.错手.contains(&组合)
                {
                    writeln!(文件, "{} {}", 码表项.name, 码表项.short).unwrap();
                }
            }
        }
        let mut 重码项: Vec<_> = 翻转码表
            .into_iter()
            .filter(|(_, (names, _))| names.len() > 1)
            .collect();
        重码项.sort_by_key(|(_, (_, frequency))| Reverse(*frequency));
        for (full, (names, frequency)) in 重码项 {
            writeln!(文件, "{full} {names:?} ({frequency})").unwrap();
        }
    }
}
