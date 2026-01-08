use chai::{
    config::{Condition, Mapped, MappedKey, 配置},
    contexts::{上下文, 合并初始决策, 展开变量, 拓扑排序},
    interfaces::默认输入,
    objectives::metric::指法标记,
    optimizers::决策,
    元素, 原始当量信息, 原始键位分布信息, 棱镜, 码表项, 编码信息, 错误,
};
use chrono::Local;
use core::panic;
use indexmap::IndexMap;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use serde_yaml::{from_str, to_string};
use std::{
    fs::{File, read_to_string},
    io::Write,
    path::PathBuf,
};

#[derive(PartialEq, Eq)]
pub enum 字源方案 {
    四码定长,
    前缀,
}
pub const 方案: 字源方案 = 字源方案::四码定长;
// pub const 大集合: [char; 21] = [
//     'b', 'p', 'm', 'f', 'd', 't', 'n', 'l', 'g', 'k', 'h', 'j', 'q', 'x', 'z', 'c', 's', 'r', 'w',
//     'y', 'v',
// ];
// pub const 小集合: [char; 5] = ['a', 'e', 'i', 'o', 'u'];
pub const 字母表: [char; 27] = [
    'b', 'p', 'm', 'f', 'd', 't', 'n', 'l', 'g', 'k', 'h', 'j', 'q', 'x', 'z', 'c', 's', 'r', 'w',
    'y', 'v', 'a', 'e', 'i', 'o', 'u', '_',
];
pub const 特简码: [char; 5] = ['a', 'e', 'i', 'o', 'u'];
pub const 特简字: [char; 5] = ['了', '的', '是', '我', '不'];
pub const 最大码长: u64 = 4;
pub const 进制: u64 = 28;
pub const 空格: u64 = 27;
pub type 频率 = u64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "类型", rename_all = "snake_case")]
pub enum 字源元素安排 {
    未选取,
    键位(char),
    归并(元素),
}

impl 字源元素安排 {
    fn from(mapped: &Mapped, 棱镜: &棱镜) -> Self {
        match mapped {
            Mapped::Unused(()) => 字源元素安排::未选取,
            Mapped::Grouped { element } => 字源元素安排::归并(棱镜.元素转数字[element]),
            Mapped::Basic(keys) => 字源元素安排::键位(
                keys.chars().next().expect("Basic 映射应至少包含一个字符"),
            ),
            Mapped::Advanced(keys) => {
                let MappedKey::Ascii(k) = &keys[0] else {
                    panic!("无法从高级映射中恢复元素安排: {:?}", mapped);
                };
                字源元素安排::键位(*k)
            }
        }
    }

    fn to_mapped(&self, 棱镜: &棱镜) -> Mapped {
        match self {
            字源元素安排::未选取 => Mapped::Unused(()),
            字源元素安排::键位(键位) => Mapped::Basic(键位.to_string()),
            字源元素安排::归并(字根) => Mapped::Grouped {
                element: 棱镜.数字转元素[&字根].clone(),
            },
        }
    }
}

#[derive(Clone)]
pub struct 字源上下文 {
    pub 配置: 配置,
    pub 棱镜: 棱镜,
    pub 初始决策: 字源决策,
    pub 决策空间: 字源决策空间,
    pub 原始键位分布信息: 原始键位分布信息,
    pub 原始当量信息: 原始当量信息,
    pub 一字信息: Vec<一字信息项>,
    pub 多字信息: Vec<多字信息项>,
    pub 动态拆分: Vec<动态拆分项>,
    pub 块转数字: FxHashMap<String, usize>,
    pub 数字转块: FxHashMap<usize, String>,
    pub 字根首笔: Vec<元素>,
    pub 字根笔画: Vec<(元素, 元素, 元素)>,
    pub 元素图: FxHashMap<元素, Vec<元素>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 字源决策 {
    pub 元素: Vec<字源元素安排>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 字源条件元素安排 {
    pub 安排: 字源元素安排,
    pub 条件列表: Vec<条件>,
    pub 打分: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 条件 {
    pub 元素: 元素,
    pub 谓词: bool,
    pub 值: 字源元素安排,
}

pub type 线性化决策 = Vec<u64>;

impl 字源决策 {
    pub fn 线性化(&self, 棱镜: &棱镜) -> 线性化决策 {
        let mut 映射 = vec![0; self.元素.len()];
        for k in 0..进制 as usize {
            映射[k] = k as u64;
        }
        for (元素, 安排) in self.元素.iter().enumerate() {
            match 安排 {
                字源元素安排::未选取 => {}
                字源元素安排::键位(键位) => {
                    映射[元素] = 棱镜.键转数字[键位];
                }
                字源元素安排::归并(元素1) => {
                    映射[元素] = 映射[*元素1];
                }
            }
        }
        映射
    }

    pub fn 允许(&self, 条件安排: &字源条件元素安排) -> bool {
        for 条件 in &条件安排.条件列表 {
            if 条件.谓词 != (self.元素[条件.元素] == 条件.值) {
                return false;
            }
        }
        return true;
    }

    pub fn 打印(&self, 棱镜: &棱镜) {
        for (元素, 安排) in self.元素.iter().enumerate() {
            if 元素 > 0 {
                println!(
                    "元素 {:?}: {:?}",
                    棱镜.数字转元素[&元素],
                    安排.to_mapped(棱镜)
                );
            }
        }
    }
}
#[derive(Debug, Clone)]
pub struct 字源决策空间 {
    pub 元素: Vec<Vec<字源条件元素安排>>,
    pub 字根: Vec<元素>,
}

#[derive(Debug, Clone)]
pub struct 字源决策变化 {
    pub 增加字根: Vec<元素>,
    pub 减少字根: Vec<元素>,
    pub 移动字根: Vec<元素>,
}

impl 字源决策变化 {
    pub fn 新建(增加: Vec<元素>, 减少: Vec<元素>, 移动: Vec<元素>) -> Self {
        字源决策变化 {
            增加字根: 增加,
            减少字根: 减少,
            移动字根: 移动,
        }
    }

    pub fn 无变化() -> Self {
        字源决策变化 {
            增加字根: vec![],
            减少字根: vec![],
            移动字根: vec![],
        }
    }
}

impl 决策 for 字源决策 {
    type 变化 = 字源决策变化;

    fn 除法(旧变化: &Self::变化, 新变化: &Self::变化) -> Self::变化 {
        let mut 移动字根 = 旧变化.移动字根.clone();
        let mut 增加字根 = 旧变化.减少字根.clone();
        let mut 减少字根 = 旧变化.增加字根.clone();
        for 元素 in &新变化.移动字根 {
            if !移动字根.contains(元素) {
                移动字根.push(*元素);
            }
        }
        for 元素 in &新变化.增加字根 {
            if !增加字根.contains(元素) {
                增加字根.push(*元素);
            }
        }
        for 元素 in &新变化.减少字根 {
            if !减少字根.contains(元素) {
                减少字根.push(*元素);
            }
        }
        Self::变化 {
            移动字根,
            增加字根,
            减少字根,
        }
    }
}

impl 上下文 for 字源上下文 {
    type 决策 = 字源决策;

    fn 序列化(&self, 解: &字源决策) -> String {
        let mut 新配置 = self.配置.clone();
        新配置.info.as_mut().unwrap().version =
            Some(format!("{}", Local::now().format("%Y-%m-%d+%H:%M:%S")));
        let mut mapping = IndexMap::new();
        for (元素, 安排) in 解.元素.iter().enumerate() {
            let mapped: Mapped = 安排.to_mapped(&self.棱镜);
            if mapped != Mapped::Unused(()) {
                mapping.insert(self.棱镜.数字转元素[&元素].clone(), mapped);
            }
        }
        新配置.form.mapping = mapping;
        to_string(&新配置).unwrap()
    }
}

#[derive(Deserialize, Clone)]
struct 原始读音 {
    拼音: String,
    频率: u64,
}

#[derive(Deserialize)]
struct 原始汉字信息 {
    汉字: char,
    gb2312: u8,
    通规: u8,
    频率: u64, // 已移至读音
    读音: Vec<原始读音>,
    字块: Vec<String>,
}

#[derive(Deserialize)]
struct 原始多字词信息 {
    词: String,
    频率: u64,
}

#[derive(Deserialize)]
struct 拆分输入 {
    汉字信息: Vec<原始汉字信息>,
    多字词信息: Vec<原始多字词信息>,
    动态拆分: FxHashMap<String, Vec<Vec<String>>>,
    字根笔画: FxHashMap<String, Vec<u8>>,
}

pub type 块 = usize;
pub type 动态拆分项 = Vec<[元素; 4]>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 一字信息项 {
    pub 词: char,
    pub 频率: 频率,
    pub 字块: [块; 4],
    pub 全拼顺取: [usize; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 多字信息项 {
    pub 词: String,
    pub 频率: 频率,
}

pub fn 对齐(列表: Vec<元素>, 默认值: 元素) -> [元素; 4] {
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

impl 字源上下文 {
    pub fn 新建(输入: 默认输入) -> Result<Self, 错误> {
        let 布局 = 输入.配置.form.clone();
        let mut 原始决策 = 布局.mapping;
        let mut 原始决策空间 = 布局.mapping_space.unwrap();
        let 原始变量映射 = 布局.mapping_variables.unwrap();
        let mut 元素转数字 = FxHashMap::default();
        let mut 数字转元素 = FxHashMap::default();
        let mut 键转数字 = FxHashMap::default();
        let mut 数字转键 = FxHashMap::default();
        for 元素 in 原始决策空间.keys() {
            if !原始决策.contains_key(元素) {
                原始决策.insert(元素.clone(), Mapped::Unused(()));
            }
        }
        合并初始决策(&mut 原始决策空间, &mut 原始决策);
        展开变量(&mut 原始决策空间, &原始变量映射);
        let (所有元素, 原始元素图) = 拓扑排序(&原始决策空间).unwrap();
        let mut 序号 = 0;
        for c in 字母表 {
            序号 += 1;
            元素转数字.insert(c.to_string(), 序号);
            数字转元素.insert(序号, c.to_string());
            键转数字.insert(c, 序号 as u64);
            数字转键.insert(序号 as u64, c);
        }
        for 元素 in &所有元素 {
            序号 += 1;
            元素转数字.insert(元素.clone(), 序号);
            数字转元素.insert(序号, 元素.clone());
        }
        let 棱镜 = 棱镜 {
            键转数字,
            数字转键,
            元素转数字,
            数字转元素,
            进制: 进制 as u64,
        };

        let 最大数量 = 棱镜.数字转元素.len() + 1;
        let mut 决策空间 = 字源决策空间 {
            元素: vec![vec![]; 最大数量],
            字根: vec![],
        };
        let mut 初始决策 = 字源决策 {
            元素: vec![字源元素安排::未选取; 最大数量],
        };
        for 元素名称 in &所有元素 {
            let 序号 = 棱镜.元素转数字[元素名称];
            if 元素名称.chars().count() == 1 {
                决策空间.字根.push(序号);
            }
            let 原始安排列表 = 原始决策空间[元素名称].clone();
            let 原始安排 = 原始决策[元素名称].clone();
            let mut 安排列表 = vec![];
            for 可行原始安排 in &原始安排列表 {
                let 可行安排 = 字源元素安排::from(&可行原始安排.value, &棱镜);
                let mut 原始条件 = 可行原始安排.condition.clone().unwrap_or_default();
                if let 字源元素安排::归并(字根) = &可行安排 {
                    let 默认条件 = Condition {
                        element: 棱镜.数字转元素[&字根].clone(),
                        op: "不是".to_string(),
                        value: Mapped::Unused(()),
                    };
                    if !原始条件.iter().any(|x| x == &默认条件) {
                        原始条件.push(默认条件);
                    }
                }
                let 条件列表: Vec<条件> = 原始条件
                    .into_iter()
                    .map(|c| 条件 {
                        元素: 棱镜.元素转数字[&c.element],
                        谓词: c.op == "是",
                        值: 字源元素安排::from(&c.value, &棱镜),
                    })
                    .collect();
                let 条件字根安排 = 字源条件元素安排 {
                    安排: 可行安排,
                    条件列表,
                    打分: 可行原始安排.score,
                };
                安排列表.push(条件字根安排);
            }
            初始决策.元素[序号] = 字源元素安排::from(&原始安排, &棱镜);
            决策空间.元素[序号] = 安排列表;
        }

        let mut 元素图 = FxHashMap::default();
        for (元素名称, 下游名称列表) in 原始元素图 {
            let 元素 = 棱镜.元素转数字[&元素名称];
            let 下游元素列表: Vec<_> = 下游名称列表
                .iter()
                .map(|name| 棱镜.元素转数字[name])
                .collect();
            元素图.insert(元素, 下游元素列表);
        }
        let (一字信息, 多字信息, 动态拆分, 块转数字, 数字转块, 字根首笔, 字根笔画) =
            Self::解析动态拆分(&棱镜, &决策空间);

        Ok(Self {
            配置: 输入.配置,
            棱镜,
            初始决策,
            决策空间,
            原始键位分布信息: 输入.原始键位分布信息,
            原始当量信息: 输入.原始当量信息,
            一字信息,
            多字信息,
            动态拆分,
            块转数字,
            数字转块,
            字根首笔,
            字根笔画,
            元素图,
        })
    }

    pub fn 解析动态拆分(
        棱镜: &棱镜,
        决策空间: &字源决策空间,
    ) -> (
        Vec<一字信息项>,
        Vec<多字信息项>,
        Vec<动态拆分项>,
        FxHashMap<String, usize>,
        FxHashMap<usize, String>,
        Vec<元素>,
        Vec<(元素, 元素, 元素)>,
    ) {
        let 拆分输入: 拆分输入 =
            from_str(&read_to_string("dynamic_analysis.yaml").unwrap()).unwrap();
        let mut 动态拆分 = vec![];
        let mut 块转数字 = FxHashMap::default();
        let mut 数字转块 = FxHashMap::default();
        let mut 字根首笔 = vec![0; 决策空间.元素.len()];
        let mut 字根笔画 = vec![(0, 0, 0); 决策空间.元素.len()];
        if 方案 == 字源方案::前缀 {
            for (字根, 笔画列表) in &拆分输入.字根笔画 {
                let 小集合笔画 = format!("补码-{}", 笔画列表[0].min(5));
                let 笔画序号 = 棱镜.元素转数字[&小集合笔画];
                let 字根序号 = 棱镜.元素转数字[字根];
                字根首笔[字根序号] = 笔画序号;
                let 第一笔 = 棱镜.元素转数字[&笔画列表[0].to_string()];
                let 第二笔 = if 笔画列表.len() > 1 {
                    棱镜.元素转数字[&笔画列表[1].to_string()]
                } else {
                    0
                };
                let 末笔 = if 笔画列表.len() > 2 {
                    棱镜.元素转数字[&笔画列表[笔画列表.len() - 1].to_string()]
                } else {
                    0
                };
                字根笔画[字根序号] = (第一笔, 第二笔, 末笔);
            }
        }
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
                let 拆分方式 = 对齐(
                    原始拆分方式
                        .iter()
                        .map(|字根| 棱镜.元素转数字[字根])
                        .collect(),
                    0,
                );
                拆分方式列表.push(拆分方式);
            }
            // 检查原始拆分方式列表的最后一项都是必选字根
            let 最后一项 = 原始拆分方式列表.last().unwrap();
            if !最后一项.iter().all(|x| {
                !决策空间.元素[棱镜.元素转数字[x]]
                    .iter()
                    .any(|x| x.安排 == 字源元素安排::未选取)
            }) {
                panic!("动态拆分方式列表的最后一项必须都是必选字根, {块:?}, {原始拆分方式列表:?}");
            }
            动态拆分.push(拆分方式列表);
        }
        let mut 一字信息 = vec![];
        let mut 多字信息 = vec![];
        let mut 合法汉字 = FxHashSet::default();
        for 词 in &拆分输入.汉字信息 {
            if !(词.gb2312 > 0 && 词.通规 > 0) {
                continue;
            }
            合法汉字.insert(词.汉字);
            let 字块 = 对齐(词.字块.iter().map(|块| 块转数字[块]).collect(), usize::MAX);
            let 拼音 = 词.读音.iter().max_by_key(|x| x.频率).cloned().unwrap().拼音;
            let mut 拼音字符: Vec<char> = 拼音.chars().collect();
            拼音字符.pop(); // 去掉声调
            let mut 全拼顺取 = [0; 3];
            if !拼音字符.is_empty() {
                // 第一码：拼音第一个字符
                全拼顺取[0] = 棱镜.键转数字[&拼音字符[0]] as 元素;
                // 第二码：如果有第二个字符就取，否则取第一个
                if 拼音字符.len() > 1 {
                    全拼顺取[1] = 棱镜.键转数字[&拼音字符[1]] as 元素;
                } else {
                    全拼顺取[1] = 全拼顺取[0];
                }
                // 第三码：拼音最后一个字符
                全拼顺取[2] = 棱镜.键转数字[&拼音字符[拼音字符.len() - 1]] as 元素;
            }
            一字信息.push(一字信息项 {
                词: 词.汉字,
                频率: 词.频率 as 频率,
                字块,
                全拼顺取,
            });
        }
        一字信息.sort_by(|a, b| b.频率.partial_cmp(&a.频率).unwrap());
        for 词 in &拆分输入.多字词信息 {
            if 词.词.chars().any(|c| !合法汉字.contains(&c)) {
                println!("跳过多字词: {}", 词.词);
                continue;
            }
            多字信息.push(多字信息项 {
                词: 词.词.clone(),
                频率: 词.频率 as 频率,
            });
        }
        多字信息.sort_by(|a, b| b.频率.partial_cmp(&a.频率).unwrap());
        (
            一字信息,
            多字信息,
            动态拆分,
            块转数字,
            数字转块,
            字根首笔,
            字根笔画,
        )
    }

    // 分析前 3000 字中全码重码和简码差指法的情况
    pub fn 分析码表(
        &self,
        编码结果: &[编码信息],
        码表: &[码表项],
        路径: &PathBuf,
    ) -> Result<(), 错误> {
        let 指法标记 = 指法标记::new();
        let mut 文件 = File::create(路径).unwrap();
        // 全码 -> 词列表的映射
        let mut 翻转码表 = FxHashMap::default();
        for (序号, 码表项) in 码表.iter().enumerate() {
            翻转码表
                .entry(码表项.full.clone())
                .or_insert_with(|| vec![])
                .push((码表项.name.clone(), 编码结果[序号].频率));
        }
        let mut 差指法 = vec![];
        for 码表项 in 码表.iter().take(2000) {
            let 词: Vec<char> = 码表项.name.chars().collect();
            let actual = if 词.len() > 1 {
                码表项.full.clone()
            } else {
                码表项.short.clone()
            };
            for 键索引 in 0..(actual.len() - 1) {
                let 组合 = (
                    actual.chars().nth(键索引).unwrap(),
                    actual.chars().nth(键索引 + 1).unwrap(),
                );
                if 指法标记.同指大跨排.contains(&组合) || 指法标记.错手.contains(&组合)
                {
                    差指法.push((码表项.name.clone(), actual.clone()));
                }
            }
        }
        let mut 重码 = vec![];
        for 码表项 in 码表.iter().take(13000) {
            let 是重码 = if 码表项.name.chars().count() > 1 {
                码表项.full_rank != 0
            } else {
                码表项.short_rank != 0
            };
            if 是重码 {
                let mut 完整重码组 = 翻转码表[&码表项.full].clone();
                let 位置 = 完整重码组
                    .iter()
                    .position(|(x, _)| x == &码表项.name)
                    .unwrap();
                完整重码组.resize(位置, ("".to_string(), 0));
                重码.push((码表项.name.clone(), 码表项.full.clone(), 完整重码组));
            }
        }
        writeln!(文件, "# 前 13000 中重码\n")?;
        for (name, code, names) in 重码 {
            writeln!(文件, "- {name} {code}：{names:?}")?;
        }
        writeln!(文件, "\n# 前 2000 中差指法项\n")?;
        for (name, code) in 差指法 {
            writeln!(文件, "- {name} {code}")?;
        }
        Ok(())
    }
}
