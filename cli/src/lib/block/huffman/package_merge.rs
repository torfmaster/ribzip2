use std::collections::HashMap;

/// Compute list of bit lengths for a given list of frequencies for symbols.
///
/// * `weights` - list of pairwise distinct frequencies of symbols
/// * `limit` - maximum bit length of code
pub(crate) fn compute_lis(weights: &[usize], limit: usize) -> Vec<usize> {
    let mut coins = vec![];
    for i in 1..=limit {
        for weight in weights {
            coins.push(Coin {
                numismatic_value: *weight,
                demonination_exponential: i,
            })
        }
    }
    let packages = package_merge(&coins);
    let solution = packages
        .iter()
        .take(weights.len() - 1)
        .flatten()
        .collect::<Vec<_>>();
    let mut code = vec![];
    for weight in weights {
        let mut counter = 0;
        for coin in solution.iter() {
            if coin.numismatic_value == *weight {
                counter += 1;
            }
        }
        code.push(counter);
    }
    code
}

/// Generic implementation for package merge for coin collector's problem with
/// binary coin values
fn package_merge(coins: &[Coin]) -> Vec<Vec<Coin>> {
    let mut map = HashMap::<usize, Vec<CoinEntry>>::new();
    let max_exp = coins
        .iter()
        .map(|x| x.demonination_exponential)
        .max()
        .unwrap();
    for i in 0..max_exp + 1 {
        map.insert(i, vec![]);
    }
    for coin in coins {
        let coin_entry = CoinEntry {
            content: Box::new(CoinEntryType::One(coin.clone())),
            numismatic_value: coin.numismatic_value,
        };
        map.get_mut(&coin.demonination_exponential)
            .unwrap()
            .push(coin_entry)
    }

    for (_, list) in map.iter_mut() {
        list.sort_by(|x, y| x.numismatic_value.cmp(&y.numismatic_value));
    }

    let mut ordered_keys = map.keys().cloned().collect::<Vec<_>>();
    ordered_keys.sort_unstable();
    ordered_keys.reverse();

    let mut coins_to_merge = Vec::<CoinEntry>::new();
    let mut last = Vec::<CoinEntry>::new();

    for demonination_exponential in ordered_keys {
        let list = map.get_mut(&demonination_exponential).unwrap();

        list.append(&mut coins_to_merge);
        list.sort_by(|x, y| x.numismatic_value.cmp(&y.numismatic_value));
        last = list.to_vec();
        let evens = list
            .iter()
            .enumerate()
            .filter(|(index, _)| index % 2 == 0)
            .map(|(_, coin)| coin);
        let odds = list
            .iter()
            .enumerate()
            .filter(|(index, _)| index % 2 == 1)
            .map(|(_, coin)| coin);
        for (left, right) in evens.zip(odds) {
            let coin_entry_to_merge = CoinEntry {
                content: Box::new(CoinEntryType::Two((left.clone(), right.clone()))),
                numismatic_value: left.numismatic_value + right.numismatic_value,
            };
            coins_to_merge.push(coin_entry_to_merge);
        }
    }
    last.into_iter()
        .map(|entry| entry.flatten())
        .collect::<Vec<_>>()
}

#[derive(Clone, Debug)]
pub(crate) struct Coin {
    pub numismatic_value: usize,
    pub demonination_exponential: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct CoinEntry {
    content: Box<CoinEntryType>,
    numismatic_value: usize,
}

impl CoinEntry {
    fn flatten(self) -> Vec<Coin> {
        match *self.content {
            CoinEntryType::One(item) => vec![item],
            CoinEntryType::Two((left, right)) => {
                let mut left = left.flatten();
                let mut right = right.flatten();
                left.append(&mut right);
                left
            }
        }
    }
}

#[derive(Clone, Debug)]
enum CoinEntryType {
    One(Coin),
    Two((CoinEntry, CoinEntry)),
}
#[cfg(test)]

mod test {

    use super::*;
    #[test]
    pub fn compute_code() {
        let res = compute_lis(&[0, 1, 2, 3, 4, 5, 6, 7], 20);
        dbg!(res);
    }

    #[test]
    pub fn creates_packages_of_value_1() {
        let res = package_merge(&[
            Coin {
                numismatic_value: 1,
                demonination_exponential: 3,
            },
            Coin {
                numismatic_value: 1,
                demonination_exponential: 3,
            },
            Coin {
                numismatic_value: 3,
                demonination_exponential: 3,
            },
            Coin {
                numismatic_value: 4,
                demonination_exponential: 3,
            },
            Coin {
                numismatic_value: 2,
                demonination_exponential: 2,
            },
            Coin {
                numismatic_value: 5,
                demonination_exponential: 2,
            },
            Coin {
                numismatic_value: 1,
                demonination_exponential: 0,
            },
            Coin {
                numismatic_value: 3,
                demonination_exponential: 0,
            },
        ]);
        for set in res {
            assert!(
                (set.iter()
                    .map(|y| 2f64.powf(-(y.demonination_exponential as f64)))
                    .sum::<f64>()
                    - 1.0)
                    .abs()
                    < f64::EPSILON
            );
        }
    }
}
