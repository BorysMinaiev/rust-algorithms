use std::cmp::min;

trait LazySegTreeSpec {
    type Elem: Clone;
    type ToPush: Clone;

    fn id() -> Self::Elem;
    fn join_pushes(p1: &mut Self::ToPush, p2: &Self::ToPush);
    fn join_elems(e1: &Self::Elem, e2: &Self::Elem) -> Self::Elem;
    fn apply_push(e: &Self::Elem, p: &Self::ToPush, l: usize, r: usize) -> Self::Elem;
    // TODO: is it possible to make this constant?
    fn no_push() -> Self::ToPush;
}

enum PlusSum {}

impl LazySegTreeSpec for PlusSum {
    type Elem = i64;
    type ToPush = i64;

    fn id() -> Self::Elem {
        0
    }

    fn join_pushes(p1: &mut Self::ToPush, p2: &Self::ToPush) {
        *p1 += p2;
    }

    fn join_elems(e1: &Self::Elem, e2: &Self::Elem) -> Self::Elem {
        e1 + e2
    }

    fn apply_push(e: &Self::Elem, p: &Self::ToPush, l: usize, r: usize) -> Self::Elem {
        e + p * (r - l) as i64
    }

    fn no_push() -> Self::ToPush {
        0
    }
}

enum PlusMin {}

impl LazySegTreeSpec for PlusMin {
    type Elem = i64;
    type ToPush = i64;

    fn id() -> Self::Elem {
        std::i64::MAX
    }

    fn join_pushes(p1: &mut Self::ToPush, p2: &Self::ToPush) {
        *p1 += p2;
    }

    fn join_elems(e1: &Self::Elem, e2: &Self::Elem) -> Self::Elem {
        min(*e1, *e2)
    }

    fn apply_push(e: &Self::Elem, p: &Self::ToPush, _l: usize, _r: usize) -> Self::Elem {
        e + p
    }

    fn no_push() -> Self::ToPush {
        0
    }
}

#[allow(dead_code)]
struct LazySegTree<T: LazySegTreeSpec> {
    values: Vec<T::Elem>,
    to_push: Vec<T::ToPush>,
    n: usize,
    relaxations: u64,
}

impl<T: LazySegTreeSpec> LazySegTree<T> {
    #[allow(dead_code)]
    fn new(init_val: T::Elem, n: usize) -> Self {
        let values = vec![init_val; n * 4];
        let to_push = vec![T::no_push(); n * 4];
        let mut res = LazySegTree { values, to_push, n, relaxations: 0 };
        res.init(0, 0, n);
        res
    }

    fn init(&mut self, v: usize, l: usize, r: usize) {
        if l + 1 == r {
            return;
        }
        let mid = (l + r) >> 1;
        self.init(v * 2 + 1, l, mid);
        self.init(v * 2 + 2, mid, r);
        self.recompute_res(v);
    }

    fn join_push(&mut self, v: usize, new_push: &T::ToPush) {
        T::join_pushes(&mut self.to_push[v], new_push);
    }

    fn relax(&mut self, v: usize) {
        self.relaxations += 1;
        let push = self.to_push[v].clone();
        self.to_push[v] = T::no_push();
        T::join_pushes(&mut self.to_push[v * 2 + 1], &push);
        T::join_pushes(&mut self.to_push[v * 2 + 2], &push);
    }

    fn recompute_res(&mut self, v: usize) {
        self.values[v] = T::join_elems(&self.get_node(v * 2 + 1), &self.get_node(v * 2 + 2));
    }

    fn apply_internal(&mut self, v: usize, need_l: usize, need_r: usize, vertex_l: usize, vertex_r: usize, change: &T::ToPush) {
        if need_l >= vertex_r || need_r <= vertex_l {
            return;
        }
        if need_l <= vertex_l && need_r >= vertex_r {
            self.join_push(v, &change);
            T::apply_push(&self.values[v], &change, vertex_l, vertex_r);
            return;
        }
        self.relax(v);
        let mid = (vertex_l + vertex_r) >> 1;
        self.apply_internal(v * 2 + 1, need_l, need_r, vertex_l, mid, change);
        self.apply_internal(v * 2 + 2, need_l, need_r, mid, vertex_r, change);
        self.recompute_res(v);
    }

    fn apply(&mut self, l: usize, r: usize, change: T::ToPush) {
        self.apply_internal(0, l, r, 0, self.n, &change);
    }

    fn get_node(&mut self, v: usize) -> T::Elem {
        self.values[v].clone()
    }

    fn get_internal(&mut self, v: usize, need_l: usize, need_r: usize, vertex_l: usize, vertex_r: usize) -> T::Elem {
        assert!(v < self.values.len());
        if need_l >= vertex_r || need_r <= vertex_l {
            return T::id();
        }
        if need_l <= vertex_l && need_r >= vertex_r {
            return self.get_node(v);
        }
        self.relax(v);
        let m = (vertex_l + vertex_r) >> 1;
        let ans_l = self.get_internal(v * 2 + 1, need_l, need_r, vertex_l, m);
        let ans_r = self.get_internal(v * 2 + 2, need_l, need_r, m, vertex_r);
        return T::join_elems(&ans_l, &ans_r);
    }

    fn get(&mut self, l: usize, r: usize) -> T::Elem {
        self.get_internal(0, l, r, 0, self.n)
    }
}


#[cfg(test)]
mod tests {
    use rand::prelude::*;
    use super::*;

    #[test]
    fn stress_plus_sum() {
        const MAX_N: usize = 50;
        const MAX_VAL: i32 = 1000_000;
        const TESTS_N: usize = 300;
        const OPS_IN_TEST: usize = 100;
        const DEBUG: bool = false;

        for t in 0..TESTS_N {
            let mut rnd = StdRng::seed_from_u64(787788 + t as u64);
            let n: usize = rnd.gen_range(1..=MAX_N);
            let mut tree = LazySegTree::<PlusSum>::new(0, n);
            let mut slow_vec = vec![0i64; n];
            if DEBUG {
                eprintln!("start test {}, n = {}", t, n);
            }
            for _ in 0..OPS_IN_TEST {
                let left = rnd.gen_range(0..n);
                let right = rnd.gen_range(left..=n);
                if rnd.gen_bool(0.5) {
                    if DEBUG {
                        eprintln!("check sum for [{}..{})", left, right);
                    }
                    let sum_from_tree = tree.get(left, right);
                    let sum_slow = slow_vec[left..right].iter().sum();
                    assert_eq!(sum_from_tree, sum_slow);
                } else {
                    let change = rnd.gen_range(0..MAX_VAL) as i64;
                    if DEBUG {
                        eprintln!("add [{}..{}) += {}", left, right, change);
                    }
                    tree.apply(left, right, change);
                    for v in &mut slow_vec[left..right] {
                        *v += change;
                    }
                }
            }
        }
    }

    #[test]
    fn stress_plus_min() {
        const MAX_N: usize = 50;
        const MAX_VAL: i32 = 1000_000;
        const TESTS_N: usize = 300;
        const OPS_IN_TEST: usize = 100;
        const DEBUG: bool = false;

        for t in 0..TESTS_N {
            let mut rnd = StdRng::seed_from_u64(787788 + t as u64);
            let n: usize = rnd.gen_range(1..=MAX_N);
            let init_val = 123;
            let mut tree = LazySegTree::<PlusMin>::new(init_val, n);
            let mut slow_vec = vec![init_val; n];
            if DEBUG {
                eprintln!("start test {}, n = {}", t, n);
            }
            for _ in 0..OPS_IN_TEST {
                let left = rnd.gen_range(0..n);
                let right = rnd.gen_range((left + 1)..=n);
                if rnd.gen_bool(0.5) {
                    if DEBUG {
                        eprintln!("check min for [{}..{})", left, right);
                    }
                    let sum_from_tree = tree.get(left, right);
                    let sum_slow = *slow_vec[left..right].iter().min().unwrap();
                    assert_eq!(sum_from_tree, sum_slow);
                } else {
                    let change = rnd.gen_range(0..MAX_VAL) as i64;
                    if DEBUG {
                        eprintln!("add [{}..{}) += {}", left, right, change);
                    }
                    tree.apply(left, right, change);
                    for v in &mut slow_vec[left..right] {
                        *v += change;
                    }
                }
            }
        }
    }


    #[test]
    fn stress_speed() {
        const MAX_N: usize = 1_000_000;
        const MAX_VAL: i32 = 1_000_000;
        const TESTS_N: usize = 10;
        const OPS_IN_TEST: usize = 1_000_000;

        for t in 0..TESTS_N {
            let mut rnd = StdRng::seed_from_u64(787788 + t as u64);
            let now = std::time::Instant::now();
            let n: usize = MAX_N;
            let init_val = 123;
            let mut tree = LazySegTree::<PlusMin>::new(init_val, n);
            for _ in 0..OPS_IN_TEST {
                let left = rnd.gen_range(0..n);
                let right = rnd.gen_range((left + 1)..=n);
                if rnd.gen_bool(0.5) {
                    tree.get(left, right);
                } else {
                    let change = rnd.gen_range(0..MAX_VAL) as i64;
                    tree.apply(left, right, change);
                }
            }
            eprintln!("total relax: {}", tree.relaxations);
            eprintln!("done with test in {}ms", now.elapsed().as_millis());
        }
    }
}