use crate::context::{
    字根安排, 字源上下文, 字源决策, 字源决策变化, 字源决策空间
};
use chai::{operators::变异, 棱镜};
use rand::{random, seq::IteratorRandom, rng};
use rustc_hash::FxHashSet;

pub struct 字源操作 {
    _棱镜: 棱镜,
    决策空间: 字源决策空间,
}

impl 变异 for 字源操作 {
    type 解类型 = 字源决策;
    fn 变异(&mut self, 决策: &mut 字源决策) -> 字源决策变化 {
        let 随机数: f64 = random();
        if 随机数 < 0.0 {
            self.交换字根(决策)
        } else if 随机数 < 0.6 {
            self.产生字根(决策)
        } else if 随机数 < 0.7 {
            self.湮灭字根(决策)
        } else {
            self.移动字根(决策)
        }
    }
}

impl 字源操作 {
    pub fn 新建(上下文: &字源上下文) -> Self {
        let 棱镜 = 上下文.棱镜.clone();
        let 决策空间 = 上下文.决策空间.clone();
        return 字源操作 {
            _棱镜: 棱镜,
            决策空间,
        };
    }

    fn 产生字根(&self, 决策: &mut 字源决策) -> 字源决策变化 {
        let mut rng = rng();
        let a: Vec<_> = 决策
            .字根
            .iter()
            .filter_map(|(字根, 安排)| {
                if *安排 != 字根安排::未选取 {
                    return None;
                }
                let 可行位置: Vec<_> = self.决策空间.字根[字根]
                    .iter()
                    .filter(|&x| match x {
                        字根安排::未选取 => false,
                        字根安排::乱序 { .. } => false,
                        字根安排::归并 {
                            字根: 被归并到字根
                        } => 决策.字根[被归并到字根] != 字根安排::未选取,
                        _ => true,
                    })
                    .cloned()
                    .collect();
                if 可行位置.is_empty() {
                    return None;
                }
                Some((字根.clone(), 可行位置))
            })
            .collect();
        if let Some((字根, 可行位置)) = a.into_iter().choose(&mut rng) {
            决策.字根[&字根] = 可行位置.into_iter().choose(&mut rng).unwrap().clone();
            字源决策变化 { 拆分改变: true }
        } else {
            字源决策变化::新建()
        }
    }

    fn 湮灭字根(&self, 决策: &mut 字源决策) -> 字源决策变化 {
        let mut rng = rng();
        let mut 可湮灭字根: FxHashSet<_> = self
            .决策空间
            .字根
            .iter()
            .filter_map(|(字根, 安排列表)| {
                if 安排列表.contains(&字根安排::未选取) {
                    Some(字根)
                } else {
                    None
                }
            })
            .collect();
        for (字根, 安排) in 决策.字根.iter() {
            if let 字根安排::归并 {
                字根: 归并到字根
            } = 安排
            {
                if 可湮灭字根.contains(归并到字根) {
                    可湮灭字根.remove(归并到字根);
                }
            } else if let 字根安排::乱序 { .. } = 安排 {
                可湮灭字根.remove(字根);
            } else if let 字根安排::未选取 = 安排 {
                可湮灭字根.remove(字根);
            }
        }
        if 可湮灭字根.is_empty() {
            return 字源决策变化::新建();
        }
        let 字根 = 可湮灭字根.into_iter().choose(&mut rng).unwrap();
        决策.字根[字根] = 字根安排::未选取;
        字源决策变化 { 拆分改变: true }
    }

    fn 移动字根(&self, 决策: &mut 字源决策) -> 字源决策变化 {
        let mut rng = rng();
        let mut 移动空间 = Vec::new();
        for (字根, 安排) in 决策.字根.iter() {
            match 安排 {
                字根安排::未选取 => continue,
                字根安排::乱序 { .. } => continue,
                其他 => {
                    let 全部安排: Vec<_> = self.决策空间.字根[字根]
                        .iter()
                        .filter(|&x| match x {
                            字根安排::未选取 => false,
                            字根安排::乱序 { .. } => false,
                            字根安排::归并 {
                                字根: 被归并到字根
                            } => 决策.字根[被归并到字根] != 字根安排::未选取,
                            a => a != 其他,
                        })
                        .cloned()
                        .collect();
                    if 全部安排.is_empty() {
                        continue;
                    }
                    移动空间.push((字根.clone(), 全部安排));
                }
            }
        }
        let (字根, 安排列表) = 移动空间.into_iter().choose(&mut rng).unwrap();
        决策.字根[&字根] = 安排列表.into_iter().choose(&mut rng).unwrap().clone();
        字源决策变化::新建()
    }

    fn 交换字根(&self, 决策: &mut 字源决策) -> 字源决策变化 {
        let mut rng = rng();
        let 字根列表: Vec<_> = 决策
            .字根
            .iter()
            .filter_map(|(k, y)| {
                if "12345".contains(k) {
                    return None;
                }
                if let 字根安排::乱序 { 键位 } = y {
                    return Some((k.clone(), *键位));
                }
                None
            })
            .collect();
        if 字根列表.len() < 2 {
            return 字源决策变化::新建();
        }
        let (字根1, 键位1) = 字根列表.iter().choose(&mut rng).unwrap();
        let (字根2, 键位2) = 字根列表.iter().choose(&mut rng).unwrap();
        if 字根1 == 字根2 {
            return 字源决策变化::新建();
        }
        决策.字根.insert(
            字根1.clone(),
            字根安排::乱序 {
                键位: 键位2.clone(),
            },
        );
        决策.字根.insert(
            字根2.clone(),
            字根安排::乱序 {
                键位: 键位1.clone(),
            },
        );
        字源决策变化::新建()
    }
}
