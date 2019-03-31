use crate::shape::AABB;
use crate::*;

pub mod nleast {

    pub struct NLeast<T> {
        n: usize,
        xs: Vec<(f32, T)>,
    }

    impl<T> NLeast<T> {
        pub fn new(n: usize) -> Self {
            NLeast { n, xs: vec![] }
        }
        pub fn insert(&mut self, x: f32, v: T) {
            if self.n == 0 {
                return;
            }

            if self.xs.len() < self.n {
                self.xs.push((x, v));
            } else if self.xs[self.n - 1].0 <= x {
                return;
            } else {
                self.xs[self.n - 1] = (x, v);
            }

            if self.xs.len() <= 1 {
                return;
            }

            for i in 1..=self.xs.len() - 1 {
                let ix1 = self.xs.len() - i;
                let ix2 = ix1 - 1;
                if self.xs[ix2].0 >= self.xs[ix1].0 {
                    self.xs.swap(ix1, ix2)
                }
            }
        }

        pub fn data(&self) -> &[(f32, T)] {
            &self.xs[..]
        }

        pub fn into_vec(self) -> Vec<(f32, T)> {
            self.xs
        }

        pub fn max_bound(&self) -> Option<f32> {
            if self.xs.len() >= self.n {
                self.xs.last().map(|x| x.0)
            } else {
                None
            }
        }
    }

    #[test]
    fn test() {
        let mut nl = NLeast::new(4);
        assert_eq!(nl.data().len(), 0);
        nl.insert(0.0, 0);
        nl.insert(-3.0, 1);
        nl.insert(2.1, 2);
        nl.insert(1.1, 3);
        {
            let data = nl.data();
            assert_eq!(data.len(), 4);
            assert_eq!(data[0], (-3.0, 1));
            assert_eq!(data[1], (0.0, 0));
            assert_eq!(data[2], (1.1, 3));
            assert_eq!(data[3], (2.1, 2));
        }
        nl.insert(5.0, 4);
        {
            let data = nl.data();
            assert_eq!(data.len(), 4);
            assert_eq!(data[0], (-3.0, 1));
            assert_eq!(data[1], (0.0, 0));
            assert_eq!(data[2], (1.1, 3));
            assert_eq!(data[3], (2.1, 2));
        }
        nl.insert(1.2, 5);
        nl.insert(-1.2, 6);
        {
            let data = nl.data();
            assert_eq!(data.len(), 4);
            assert_eq!(data[0], (-3.0, 1));
            assert_eq!(data[1], (-1.2, 6));
            assert_eq!(data[2], (0.0, 0));
            assert_eq!(data[3], (1.1, 3));
        }
    }
}

pub struct KDTree<T> {
    data: Vec<T>,
    coords: Vec<P3>,
    nodes: Vec<KDTreeNode>,
}

#[derive(Debug)]
struct KDTreeNode {
    value_index: usize,
    //split_axis: usize,
    left: Option<(AABB, usize)>,
    right: Option<(AABB, usize)>,
}

impl<T> KDTree<T> {
    pub fn new<F>(data: Vec<T>, f: F) -> Self
    where
        F: Fn(&T) -> P3,
    {
        let coords = data.iter().map(f).collect::<Vec<_>>();
        let mut nodes = vec![];
        let mut ixs = (0..coords.len()).collect::<Vec<_>>();
        Self::build(&mut nodes, &mut ixs, &coords);
        KDTree {
            data,
            coords,
            nodes,
        }
    }

    fn build(
        nodes: &mut Vec<KDTreeNode>,
        ixs: &mut [usize],
        coords: &Vec<P3>,
    ) -> Option<(AABB, usize)> {
        if ixs.is_empty() {
            return None;
        }
        let aabb = ixs
            .iter()
            .fold(AABB::single_point(&coords[ixs[0]]), |b, ix| {
                b.merge(&AABB::single_point(&coords[*ix]))
            });
        let split_axis = aabb.longest_axis();
        ixs.sort_by(|ix1, ix2| {
            let v1 = coords[*ix1][split_axis];
            let v2 = coords[*ix2][split_axis];
            v1.partial_cmp(&v2).unwrap()
        });

        let n_left = (ixs.len() - 1) / 2;
        let value_index = n_left;

        let new_node_index = nodes.len();
        nodes.push(KDTreeNode {
            value_index: ixs[value_index],
            //split_axis,
            left: None,
            right: None,
        });

        nodes[new_node_index].left = Self::build(nodes, &mut ixs[..n_left], coords);
        nodes[new_node_index].right = Self::build(nodes, &mut ixs[n_left + 1..], coords);
        Some((aabb, new_node_index))
    }

    pub fn n_nearest(&self, p: &P3, n: usize, max_distance: f32) -> Vec<(f32, &T)> {
        let mut list = nleast::NLeast::new(n);
        let mut stack = vec![0];
        while let Some(node) = stack.pop() {
            let node = &self.nodes[node];
            let value_index = node.value_index;
            let distance = (self.coords[value_index] - p).norm();
            if distance <= max_distance {
                list.insert(distance, value_index);
            }

            if let Some((aabb, ix)) = &node.left {
                let min_dist = aabb.min_distance_from(p);
                if min_dist <= max_distance && min_dist <= list.max_bound().unwrap_or(std::f32::MAX)
                {
                    stack.push(*ix);
                }
            }

            if let Some((aabb, ix)) = &node.right {
                let min_dist = aabb.min_distance_from(p);
                if min_dist <= max_distance && min_dist <= list.max_bound().unwrap_or(std::f32::MAX)
                {
                    stack.push(*ix);
                }
            }
        }

        list.data()
            .iter()
            .map(|(d, i)| (*d, &self.data[*i]))
            .collect()
    }
}

#[test]
fn test() {
    let mut data = {
        use rand::distributions::Uniform;
        use rand::prelude::*;
        use rand::thread_rng;
        let mut rng = thread_rng();
        let distribution = Uniform::new(-100.0, 100.0);
        (0..100)
            .map(|i| {
                let x = distribution.sample(&mut rng);
                let y = distribution.sample(&mut rng);
                let z = distribution.sample(&mut rng);
                (i, P3::new(x, y, z))
            })
            .collect::<Vec<_>>()
    };
    let p = P3::new(0.0, 0.0, 0.0);
    let n = 8;
    let kd = KDTree::new(data.clone(), |x| x.1);
    let nearest = kd.n_nearest(&p, n, 100.0);
    assert_eq!(nearest.len(), data.len().min(n));
    data.sort_by(|v1, v2| {
        let dist1 = (v1.1 - p).norm();
        let dist2 = (v2.1 - p).norm();
        dist1.partial_cmp(&dist2).unwrap()
    });
    for (v1, (_, v2)) in data.iter().zip(nearest.iter()) {
        assert_eq!(v1.0, v2.0);
    }
}
