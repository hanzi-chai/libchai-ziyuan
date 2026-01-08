use crate::context::{
    字源上下文, 字源元素安排, 字源决策, 字源决策变化, 字源决策空间
};
use chai::{operators::变异, 元素, 棱镜};
use rand::{random, random_range, rng, seq::IndexedRandom};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

const 最大尝试次数: usize = 100;

pub struct 字源操作 {
    _棱镜: 棱镜,
    决策空间: 字源决策空间,
    下游字根: FxHashMap<元素, Vec<元素>>,
}

impl 变异 for 字源操作 {
    type 决策 = 字源决策;
    fn 变异(&mut self, 决策: &mut 字源决策) -> 字源决策变化 {
        let 随机数: f64 = random();
        let mut 变化 = if 随机数 < 0.2 {
            self.产生字根(决策)
        } else if 随机数 < 0.4 {
            self.湮灭字根(决策)
        } else {
            self.移动字根(决策)
        };
        self.传播(&mut 变化, 决策);
        变化
    }
}

impl 字源操作 {
    pub fn 新建(上下文: &字源上下文) -> Self {
        let 棱镜 = 上下文.棱镜.clone();
        let 决策空间 = 上下文.决策空间.clone();
        let 下游字根 = 上下文.元素图.clone();
        return 字源操作 {
            _棱镜: 棱镜,
            决策空间,
            下游字根,
        };
    }

    fn 传播(&self, 变化: &mut 字源决策变化, 决策: &mut 字源决策) {
        // 初始化队列
        let mut 队列 = VecDeque::new();
        for 元素 in 变化
            .增加字根
            .iter()
            .chain(变化.减少字根.iter())
            .chain(变化.移动字根.iter())
        {
            for 下游元素 in self.下游字根.get(元素).unwrap_or(&vec![]) {
                if !队列.contains(下游元素) {
                    队列.push_back(下游元素.clone());
                }
            }
        }
        let mut iters = 0;
        while !队列.is_empty() {
            iters += 1;
            if iters > 100 {
                panic!(
                    "传播超过 100 次仍未结束，可能出现死循环，当前队列为：{:?}",
                    队列
                        .iter()
                        .map(|x| &self._棱镜.数字转元素[&x])
                        .collect::<Vec<_>>()
                );
            }
            let 元素 = 队列.pop_front().unwrap();
            let mut 合法 = false;
            let mut 新安排列表 = vec![];
            for 条件安排 in &self.决策空间.元素[元素] {
                if 决策.允许(条件安排) {
                    if 条件安排.安排 == 决策.元素[元素] {
                        合法 = true;
                        break;
                    }
                    新安排列表.push(条件安排.安排.clone());
                }
            }
            if !合法 {
                if 新安排列表.is_empty() {
                    let 元素字符串 = &self._棱镜.数字转元素[&元素];
                    panic!(
                        "{元素字符串:?} 没有合法的安排，传播失败，全部空间为 {:?}",
                        self.决策空间.元素[元素]
                    );
                } else {
                    let 新安排 = 新安排列表.choose(&mut rng()).unwrap();
                    if 决策.元素[元素] == 字源元素安排::未选取 {
                        变化.增加字根.push(元素);
                    } else if 新安排 == &字源元素安排::未选取 {
                        变化.减少字根.push(元素);
                    } else {
                        变化.移动字根.push(元素);
                    }
                    决策.元素[元素] = 新安排.clone();
                }
            }
            for 下游元素 in self.下游字根.get(&元素).unwrap_or(&vec![]) {
                if !队列.contains(下游元素) {
                    队列.push_back(下游元素.clone());
                }
            }
        }
    }

    fn 产生字根(&self, 决策: &mut 字源决策) -> 字源决策变化 {
        let mut rng = rng();
        for _ in 0..最大尝试次数 {
            let 元素 = *self.决策空间.字根.choose(&mut rng).unwrap();
            if 决策.元素[元素] != 字源元素安排::未选取 {
                continue;
            }
            // 蓄水池抽样
            let mut 下一个安排 = None;
            let mut count = 0;
            for 条件安排 in &self.决策空间.元素[元素] {
                if 条件安排.安排 != 字源元素安排::未选取 && 决策.允许(条件安排)
                {
                    count += 1;
                    if random_range(0..count) == 0 {
                        下一个安排 = Some(条件安排.安排);
                    }
                }
            }
            if let Some(下一个安排) = 下一个安排 {
                决策.元素[元素] = 下一个安排;
                return 字源决策变化::新建(vec![元素], vec![], vec![]);
            }
        }
        字源决策变化::无变化()
    }

    fn 湮灭字根(&self, 决策: &mut 字源决策) -> 字源决策变化 {
        let mut rng = rng();
        for _ in 0..最大尝试次数 {
            let 元素 = *self.决策空间.字根.choose(&mut rng).unwrap();
            if 决策.元素[元素] == 字源元素安排::未选取 {
                continue;
            }
            for 条件安排 in &self.决策空间.元素[元素] {
                if 条件安排.安排 == 字源元素安排::未选取 && 决策.允许(条件安排)
                {
                    决策.元素[元素] = 条件安排.安排;
                    return 字源决策变化::新建(vec![], vec![], vec![元素]);
                }
            }
        }
        字源决策变化::无变化()
    }

    fn 移动字根(&self, 决策: &mut 字源决策) -> 字源决策变化 {
        let mut rng = rng();
        for _ in 0..最大尝试次数 {
            let 元素 = *self.决策空间.字根.choose(&mut rng).unwrap();
            if 决策.元素[元素] == 字源元素安排::未选取 {
                continue;
            }
            // 蓄水池抽样
            let mut 下一个安排 = None;
            let mut count = 0;
            for 条件安排 in &self.决策空间.元素[元素] {
                if 条件安排.安排 != 决策.元素[元素]
                    && 条件安排.安排 != 字源元素安排::未选取
                    && 决策.允许(条件安排)
                {
                    count += 1;
                    if random_range(0..count) == 0 {
                        下一个安排 = Some(条件安排.安排);
                    }
                }
            }
            if let Some(下一个安排) = 下一个安排 {
                决策.元素[元素] = 下一个安排;
                return 字源决策变化::新建(vec![], vec![], vec![元素]);
            }
        }
        字源决策变化::无变化()
    }
}
