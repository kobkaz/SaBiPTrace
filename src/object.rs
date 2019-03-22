use crate::*;

fn merge_options<T, F: Fn(T, T) -> T>(x: Option<T>, y: Option<T>, merge: F) -> Option<T> {
    if let Some(x) = x {
        if let Some(y) = y {
            Some(merge(x, y))
        } else {
            Some(x)
        }
    } else {
        y
    }
}

#[derive(Clone, Debug)]
pub struct ObjectHit {
    pub geom: shape::Hit,
    pub material: material::Material,
    pub emission: Option<RGB>,
    pub obj_ix: usize,
}

impl ObjectHit {
    fn nearer_option(x: Option<Self>, y: Option<Self>) -> Option<Self> {
        merge_options(x, y, |x, y| if x.geom.dist < y.geom.dist { x } else { y })
    }

    pub fn pos(&self) -> &P3 {
        &self.geom.pos
    }
}

pub struct SimpleObject {
    pub shape: shape::Shape,
    pub material: material::Material,
    pub emission: Option<RGB>,
}

impl SimpleObject {
    pub fn test_hit(&self, ray: &Ray, tnear: f32, tfar: f32, self_ix: usize) -> Option<ObjectHit> {
        self.shape.test_hit(ray, tnear, tfar).map(|geom| ObjectHit {
            geom,
            material: self.material.clone(),
            emission: self.emission,
            obj_ix: self_ix,
        })
    }
}

pub struct ObjectList {
    pub objects: Vec<SimpleObject>,
}

impl ObjectList {
    pub fn new() -> Self {
        ObjectList { objects: vec![] }
    }

    pub fn test_hit(&self, ray: &Ray, tnear: f32, mut tfar: f32) -> Option<ObjectHit> {
        let mut hit = None::<ObjectHit>;
        for (i, o) in self.objects.iter().enumerate() {
            tfar = hit.as_ref().map_or(tfar, |h| h.geom.dist);
            let new_hit = o.test_hit(ray, tnear, tfar, i);
            hit = ObjectHit::nearer_option(hit, new_hit);
        }
        hit
    }
}

#[derive(Clone)]
enum BVHNode {
    Leaf {
        aabb: shape::AABB,
        object_ix: usize,
    },
    Node {
        aabb: shape::AABB,
        l_child: usize,
        r_child: usize,
    },
}

pub struct BVH {
    objects: Vec<SimpleObject>,
    tree: Vec<BVHNode>,
}

impl BVH {
    pub fn new(objects: Vec<SimpleObject>) -> Self {
        let l = objects.len();
        let mut ixs: Vec<_> = (0..l).map(|i| (i, objects[i].shape.aabb())).collect();
        let dummy_leaf = BVHNode::Leaf {
            aabb: shape::AABB::new(&P3::origin(), &P3::origin()),
            object_ix: l,
        };
        let mut tree = vec![dummy_leaf; 2 * objects.len() - 1];
        BVH::build(&mut tree, 0, &mut ixs, 0, l);
        BVH { objects, tree }
    }
    pub fn objects(&self) -> &Vec<SimpleObject> {
        &self.objects
    }

    fn build(
        nodes: &mut Vec<BVHNode>,
        next_node_ix: usize,
        ixs: &mut Vec<(usize, shape::AABB)>,
        begin: usize,
        end: usize,
    ) -> usize {
        if end <= begin + 1 {
            nodes[next_node_ix] = BVHNode::Leaf {
                aabb: ixs[begin].1.clone(),
                object_ix: ixs[begin].0,
            };
            return next_node_ix + 1;
        } else {
            let aabb = {
                let mut aabb = ixs[begin].1.clone();
                for i in begin + 1..end {
                    aabb = aabb.merge(&ixs[i].1)
                }
                aabb
            };

            //sort ixs
            {
                let axis = aabb.diag().iamax();
                let ixs_slice = &mut ixs[begin..end];
                ixs_slice.sort_by(|(_, ab0), (_, ab1)| {
                    ab0.center()[axis].partial_cmp(&ab1.center()[axis]).unwrap()
                });
            }

            let mid = (end + begin) / 2;
            let node_ix = next_node_ix;
            let l_child = node_ix + 1;
            let r_child = BVH::build(nodes, l_child, ixs, begin, mid);
            let next_node_ix = BVH::build(nodes, r_child, ixs, mid, end);
            nodes[node_ix] = BVHNode::Node {
                aabb,
                l_child,
                r_child,
            };
            return next_node_ix;
        }
    }

    pub fn test_hit(&self, ray: &Ray, tnear: f32, tfar: f32) -> Option<ObjectHit> {
        self.test_hit_search(0, ray, tnear, tfar)
    }

    fn test_hit_search(
        &self,
        node_ix: usize,
        ray: &Ray,
        tnear: f32,
        tfar: f32,
    ) -> Option<ObjectHit> {
        match self.tree[node_ix] {
            BVHNode::Leaf { object_ix, .. } => {
                self.objects[object_ix].test_hit(ray, tnear, tfar, object_ix)
            }
            BVHNode::Node {
                ref aabb,
                l_child,
                r_child,
            } => aabb
                .ray_intersect(ray, tnear, tfar)
                .and_then(|(tnear, tfar)| {
                    let hit_l = self.test_hit_search(l_child, ray, tnear, tfar);
                    let hit_r = self.test_hit_search(r_child, ray, tnear, tfar);
                    ObjectHit::nearer_option(hit_l, hit_r)
                }),
        }
    }
}
