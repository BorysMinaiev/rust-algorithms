#[allow(dead_code)]
pub struct Fenwick {
    values: Vec<i64>
}

impl Fenwick {
    #[allow(dead_code)]
    fn get_sum(&self, mut pos: usize) -> i64 {
        let mut res = 0i64;
        loop {
            res += self.values[pos] as i64;
            pos = pos & (pos + 1);
            if pos == 0 {
                return res;
            }
            pos -= 1;
        }
    }

    #[allow(dead_code)]
    fn add(&mut self, mut pos: usize, change: i64) {
        while pos < self.values.len() {
            self.values[pos] += change;
            pos |= pos + 1;
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new(n: usize) -> Self {
        let values = vec![0; n];
        Fenwick { values }
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::*;
    use super::*;

    #[test]
    fn stress() {
        let mut rnd = StdRng::from_rng(thread_rng()).unwrap();
        const MAX_N: usize = 100;
        const MAX_VAL: i32 = std::i32::MAX;
        const TESTS_N: usize = 100;

        for _ in 0..TESTS_N {
            let n: usize = rnd.gen_range(1..=MAX_N);
            let mut fenw = Fenwick::new(n);
            let mut slow_vec = vec![0i64; n];
            for _ in 0..TESTS_N {
                let pos = rnd.gen_range(0..n);
                if rnd.gen_bool(0.5) {
                    let sum_from_fenw = fenw.get_sum(pos);
                    let sum_slow = slow_vec[0..=pos].iter().sum();
                    assert_eq!(sum_from_fenw, sum_slow);
                } else {
                    let change = rnd.gen_range(0..MAX_VAL) as i64;
                    fenw.add(pos, change);
                    slow_vec[pos] += change;
                }
            }
        }
    }

    #[test]
    fn stress_speed() {
        const MAX_N: usize = 1_000_000;
        const MAX_VAL: i32 = 1_000_000;
        const TESTS_N: usize = 1;
        const OPS_IN_TEST: usize = 20_000_000;

        for t in 0..TESTS_N {
            let mut rnd = StdRng::seed_from_u64(787788 + t as u64);
            let now = std::time::Instant::now();
            let n: usize = MAX_N;
            let mut tree = Fenwick::new(n);
            let mut tot_sum = 0;
            for _ in 0..OPS_IN_TEST {
                let pos = rnd.gen_range(0..n);
                if rnd.gen_bool(0.5) {
                    tot_sum += tree.get_sum(pos);
                } else {
                    let change = rnd.gen_range(0..MAX_VAL) as i64;
                    tree.add(pos, change);
                }
            }
            eprintln!("hash val: {}", tot_sum);
            eprintln!("done with test in {}ms", now.elapsed().as_millis());
        }
    }
}