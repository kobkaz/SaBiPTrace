use super::*;
use scene::Scene;
use std::borrow::Cow::{Owned, Borrowed};

#[allow(unused_variables)]
fn strategy_weight(s: usize, t: usize) -> Option<f32> {
    Some(1.0)
}

#[derive(Clone)]
pub struct Vertex {
    pub hit: object::ObjectHit,
    pub throughput: RGB,
    pub w_local: V3,
    pub pdf_area: f32,
    pub pdf_area_ratio: f32,
    pub specular: bool,
}

impl Vertex {
    pub fn pos(&self) -> &P3 {
        &self.hit.geom.pos
    }

    pub fn gnorm(&self) -> &V3 {
        &self.hit.geom.gnorm
    }
}

#[derive(Debug, Clone)]
struct ExtVertex {
    pdf_area: f32,
    pdf_area_ratio: f32,
    all_specular: bool,
}

impl ExtVertex {
    pub fn new(pdf_area: f32, pdf_area_ratio: f32, all_specular: bool) -> Self {
        ExtVertex {
            pdf_area,
            pdf_area_ratio,
            all_specular,
        }
    }
}

fn continue_chance_from_throughput(throughput: &RGB, depth: usize) -> f32 {
    if !throughput.is_finite() {
        return 0.0;
    }

    let base = (throughput.max() * 0.8).min(1.0).max(0.0);
    if depth < 10 {
        base
    } else {
        base * (0.95f32.powi(depth as i32 - 10))
    }
}

pub fn gen_vertices<R: ?Sized>(
    scene: &Scene,
    ray: &Ray,
    init_ray_delta: bool,
    max_depth: usize,
    rng: &mut R,
) -> Vec<Vertex>
where
    R: Rng,
{
    let mut vs: Vec<Vertex> = vec![];
    let mut throughput = RGB::all(1.0);
    let mut ray = ray.clone();
    let mut pdf_area = 1.0;
    let mut pdf_area_ratio = 1.0;
    for depth in 0..max_depth {
        let hit = scene.test_hit(&ray, 1e-3, std::f32::MAX / 2.0);
        if hit.is_none() {
            break;
        }
        let hit = hit.unwrap();
        let hit_lc = hit.geom.lc();
        let wout_local = hit_lc.w2l() * -ray.dir;
        if vs.last().map(|v| !v.specular).unwrap_or(!init_ray_delta) {
            //convert the solid-angle pdf from the previous loop
            pdf_area *= wout_local[2].abs() / hit.geom.dist / hit.geom.dist;
            pdf_area_ratio *= wout_local[2].abs() / hit.geom.dist / hit.geom.dist;
        }

        let next = hit.material.sample_win_cos(&wout_local, rng);
        let win_local = next.value.0;

        vs.push(Vertex {
            hit: hit.clone(),
            throughput,
            w_local: wout_local,
            pdf_area,
            pdf_area_ratio,
            specular: next.value.2,
        });
        pdf_area_ratio = 1.0;

        let bsdf_cos = next.value.1;
        throughput *= bsdf_cos / next.pdf;
        // will be converted into area pdf in the next loop
        pdf_area *= next.pdf;
        pdf_area_ratio *= next.pdf;

        let cont = pdf::RandomBool {
            chance: continue_chance_from_throughput(&throughput, depth),
        };
        let cont = cont.sample(rng);
        if !cont.value {
            break;
        }

        throughput /= cont.pdf;
        pdf_area *= cont.pdf;
        pdf_area_ratio *= cont.pdf;
        ray = hit_lc.l2w() * Ray::new(P3::origin(), win_local);
    }
    vs
}

fn extend_path_pdf<'a>(
    ray_delta: bool,
    origin: Option<&P3>,
    vs_init: &'a [Vertex],
    vs_latter: &'a [Vertex],
    v_last: Option<(P3, V3, bool)>,
) -> impl Iterator<Item = ExtVertex> + 'a {
    use either::Either::{Left, Right};
    if origin.is_none() {
        assert!(vs_init.is_empty());
    }
    {
        let mut vs = if origin.is_none() { 0 } else { 1 };
        vs += if v_last.is_none() { 0 } else { 1 };
        vs += vs_init.len();
        vs += vs_latter.len();
        assert!(vs >= 2);
    }

    //if origin is None or vs_init is empty, make them from vs_latter
    let (origin, vs_init, vs_latter, connection_vertices) = if !vs_init.is_empty() {
        (origin.unwrap(), Left(vs_init), vs_latter, 2)
    } else {
        let (origin, pseudo_init_original, latter, connection_vertices) =
            if let Some(origin) = origin {
                (
                    origin,
                    &vs_latter[vs_latter.len() - 1],
                    &vs_latter[0..vs_latter.len() - 1],
                    1,
                )
            } else {
                (
                    vs_latter[vs_latter.len() - 1].pos(),
                    &vs_latter[vs_latter.len() - 2],
                    &vs_latter[0..vs_latter.len() - 2],
                    0,
                )
            };
        let w = origin - pseudo_init_original.pos();
        let r = w.norm();
        let w_local = (pseudo_init_original.hit.geom.lc().w2l() * w).normalize();
        let pdf_area = if ray_delta {
            1.0
        } else {
            w_local[2].abs() / r / r
        };

        let pseudo_init = Vertex {
            hit: pseudo_init_original.hit.clone(),
            throughput: RGB::all(1.0),
            specular: pseudo_init_original.specular,
            w_local,
            pdf_area,
            pdf_area_ratio: pdf_area,
        };
        (origin, Right(pseudo_init), latter, connection_vertices)
    };

    struct State {
        depth: usize,
        dir: V3,
        throughput: RGB,
        pdf_area: f32,
        connection_vertices: usize,
    };

    let (terminal_vertex, vs_init_len) = match vs_init {
        Left(vs) => (vs.last().unwrap(), vs.len()),
        Right(ref v) => (v, 1),
    };

    let init_state = {
        let terminal_prev = match vs_init {
            Left(vs) => {
                if vs.len() >= 2 {
                    vs[vs.len() - 2].pos()
                } else {
                    origin
                }
            }
            _ => origin,
        };
        let init_state = State {
            depth: vs_init_len - 1,
            dir: (terminal_prev - terminal_vertex.pos()).normalize(),
            throughput: terminal_vertex.throughput,
            pdf_area: terminal_vertex.pdf_area,
            connection_vertices,
        };
        init_state
    };

    let reflection_vertices = Some(Owned(terminal_vertex.clone()))
        .into_iter()
        .chain(vs_latter.iter().rev().map(Borrowed));
    let target_vertices = vs_latter
        .iter()
        .map(|v| (*v.pos(), *v.gnorm(), v.hit.material.all_specular()))
        .rev()
        .chain(v_last);

    let before_connection = match vs_init {
        Left(vs_init) => {
            let vs = vs_init.iter().map(|v| {
                ExtVertex::new(v.pdf_area, v.pdf_area_ratio, v.hit.material.all_specular())
            });
            Left(vs)
        }
        Right(ref v) => {
            let v = ExtVertex::new(v.pdf_area, v.pdf_area_ratio, v.hit.material.all_specular());
            let once = std::iter::once(v);
            Right(once)
        }
    };

    assert!(init_state.throughput.max() >= 0.0);
    let latter =
        reflection_vertices
            .zip(target_vertices)
            .scan(init_state, move |state, (v, next)| {
                let (next_pos, next_normal, next_all_specular) = next;
                let v_geom = &v.hit.geom;
                let lc = v_geom.lc();
                let wout_local = lc.w2l() * state.dir;
                let to_next = next_pos - v_geom.pos;
                let r = to_next.norm();
                let win = to_next / r;
                let win_local = lc.w2l() * win;
                if state.connection_vertices > 0 {
                    if v.hit.material.all_specular() {
                        panic!("connecting specular {} {}", vs_init_len, vs_latter.len());
                    }
                }
                let specular_component = state.connection_vertices == 0 && v.specular;
                let dir_pdf_omega =
                    v.hit
                        .material
                        .sample_win_pdf(&wout_local, &win_local, specular_component);
                let bsdf_cos = v
                    .hit
                    .material
                    .bsdf_cos(&win_local, &wout_local, specular_component);
                assert!(bsdf_cos.max() >= 0.0);
                assert!(dir_pdf_omega >= 0.0);

                let mut pdf_area_ratio = 1.0;
                state.throughput *= bsdf_cos / dir_pdf_omega;
                state.pdf_area *= dir_pdf_omega;
                pdf_area_ratio *= dir_pdf_omega;
                if !specular_component {
                    state.pdf_area *= win.dot(&next_normal).abs() / r / r;
                    pdf_area_ratio *= win.dot(&next_normal).abs() / r / r;
                }
                let cont_prob = continue_chance_from_throughput(&state.throughput, state.depth);
                state.throughput /= cont_prob;
                state.pdf_area *= cont_prob;
                pdf_area_ratio *= cont_prob;
                state.depth += 1;
                state.dir = -win;
                if state.connection_vertices > 0 {
                    state.connection_vertices -= 1;
                }

                if pdf_area_ratio > 0.0 {
                    Some(ExtVertex {
                        pdf_area: state.pdf_area,
                        pdf_area_ratio,
                        all_specular: next_all_specular,
                    })
                } else {
                    None
                }
            });

    before_connection.chain(latter)
}

fn mis_weight(
    scene: &Scene,
    ray: &Ray,
    eye_vs: &[Vertex],
    light_vs: &[Vertex],
    light_sample: Option<&scene::LightSampleResult>,
) -> f32 {
    let original_s = if light_sample.is_none() {
        assert!(light_vs.is_empty());
        0
    } else {
        light_vs.len() + 1
    };
    let original_t = eye_vs.len() + 1;
    assert!(strategy_weight(original_s, original_t).is_some());
    assert!(!eye_vs.is_empty());
    assert!(original_t >= 2);
    if !eye_vs.is_empty() && original_s != 0 {
        assert!(!eye_vs.last().unwrap().hit.material.all_specular())
    }
    if !light_vs.is_empty() && original_t != 0 {
        assert!(!light_vs.last().unwrap().hit.material.all_specular())
    }

    let eye_extended: Vec<_> = extend_path_pdf(
        true,
        Some(&ray.origin),
        eye_vs,
        light_vs,
        light_sample.map(|ls| (ls.pos, ls.normal, false)),
    )
    .collect();
    let mut light_pos_pdf = 1.0;
    let mut light_dir_pdf = 1.0;

    let light_extended: Vec<_> = if original_s == 0 {
        assert!(light_sample.is_none());
        if original_t == 2 {
            vec![]
        } else {
            let light = eye_vs.last().unwrap();
            let light_pos = light.pos();
            assert!(light.hit.emission.is_some());
            light_pos_pdf *= scene.sample_light_pdf(&light_pos, light.hit.obj_ix);
            light_dir_pdf *= 1.0; // TODO direction pdf

            extend_path_pdf(false, None, &[], &eye_vs, None).collect()
        }
    } else if original_s == 1 {
        assert!(light_vs.is_empty());
        let light = light_sample.unwrap();
        light_pos_pdf *= scene.sample_light_pdf(&light.pos, light.obj_ix);
        light_dir_pdf *= 1.0; // TODO direction pdf

        extend_path_pdf(false, Some(&light.pos), &[], &eye_vs, None).collect()
    } else {
        let light = light_sample.unwrap();
        light_pos_pdf *= scene.sample_light_pdf(&light.pos, light.obj_ix);
        light_dir_pdf *= 1.0; // TODO direction pdf

        extend_path_pdf(false, Some(&light.pos), &light_vs, &eye_vs, None).collect()
    };

    let pdfs_r: Vec<f32> = (0..=original_s + original_t - 2)
        .scan(1.0, |r_pdf, s| {
            let t = original_s + original_t - s;
            assert!(t >= 2);

            let c = strategy_weight(s, t).unwrap_or(0.0);

            if t >= eye_extended.len() + 2 {
                return Some(0.0);
            } else if t == eye_extended.len() + 1 {
                if s != 0 && eye_extended[t - 2].all_specular {
                    return Some(0.0);
                } else {
                    return Some(c * *r_pdf);
                }
            }

            *r_pdf /= eye_extended[t - 1].pdf_area_ratio;
            if s == 1 {
                *r_pdf *= light_pos_pdf;
            } else if s == 2 {
                *r_pdf *= light_dir_pdf;
            }

            if s >= 2 {
                if s >= light_extended.len() + 2 {
                    return Some(0.0);
                }
                *r_pdf *= light_extended[s - 2].pdf_area_ratio;
            }

            let e_specular = eye_extended[t - 2].all_specular;
            let l_specular = if s < 2 {
                //assume no directional light
                false
            } else {
                light_extended[s - 2].all_specular
            };
            if s != 0 && (e_specular || l_specular) {
                Some(0.0)
            } else {
                Some(c * *r_pdf)
            }
        })
        .collect();
    let sum_r = pdfs_r.iter().sum::<f32>();
    let original_pdf_r = pdfs_r[original_s];
    let weight_r = original_pdf_r / sum_r;
    if !weight_r.is_finite() {
        0.0
    } else {
        weight_r
    }
}

pub fn radiance<R: ?Sized>(
    scene: &Scene,
    ray: &Ray,
    radiance_accum: &mut impl Accumulator,
    rng: &mut R,
) where
    R: Rng,
{
    //TODO: take acount of depth limits in MIS
    const LE_MAX: usize = 30;
    const LL_MAX: usize = 30;
    let eye_vs = gen_vertices(scene, ray, true, LE_MAX, rng);
    let len_e = eye_vs.len();

    let light_sample = scene.sample_light(rng);
    if light_sample.is_none() {
        return;
    }
    let light_sample = light_sample.unwrap();

    let initial_ray = light_sample.as_ref().and_then(
        |scene::LightSampleResult {
             pos: light_pos,
             normal: light_normal,
             emission: light_emission,
             ..
         }| {
            pdf::CosUnitHemisphere::from_normal(light_normal)
                .sample(rng)
                .and_then(|v| {
                    pdf::RandomBool { chance: 0.5 }
                        .sample(rng)
                        .map(|b| if b { -v } else { v })
                })
                .map(move |initial_outdir| {
                    let light_emission_cos =
                        *light_emission * initial_outdir.dot(light_normal).abs();
                    let initial_ray = Ray::new(*light_pos, initial_outdir);
                    (initial_ray, light_emission_cos)
                })
        },
    );

    let light_vs = gen_vertices(scene, &initial_ray.value.0, false, LL_MAX, rng);
    let len_l = light_vs.len();

    for len in 2..=len_e + len_l + 4 {
        let s_min = len - len.min(LE_MAX + 2);
        let s_max = (len - 2).min(LL_MAX + 2);
        assert!(s_min <= s_max);
        let mut accum_len = RGB::all(0.0);
        for s in s_min..=s_max {
            let t = len - s;
            assert!(t >= 2);
            if strategy_weight(s, t).is_none() {
                continue;
            }
            let e_i = t - 2;
            if e_i >= len_e {
                continue;
            }
            let v_eye = &eye_vs[e_i];

            let (contrib, mis_weight) = if s == 0 {
                if let Some(emission) = v_eye.hit.emission {
                    let mis_weight = mis_weight(scene, ray, &eye_vs[0..t - 1], &[], None);
                    (emission * v_eye.throughput, mis_weight)
                } else {
                    (RGB::all(0.0), 0.0)
                }
            } else if s == 1 {
                let Vertex {
                    hit,
                    throughput,
                    w_local: wout_local,
                    ..
                } = v_eye;
                let hit_lc = hit.geom.lc();
                let scene::LightSampleResult {
                    pos: ref light_pos,
                    normal: ref light_normal,
                    emission: light_emission,
                    ..
                } = light_sample.value;
                if v_eye.hit.material.all_specular() {
                    continue;
                } else if !scene.visible(light_pos, hit.pos()) {
                    continue;
                } else {
                    let g = hit.geom.g(light_pos, light_normal);
                    let light_dir = (light_pos - hit.pos()).normalize();
                    let bsdf = hit
                        .material
                        .bsdf(&(hit_lc.w2l() * light_dir), &wout_local, false);
                    let mis_weight = mis_weight(
                        scene,
                        ray,
                        &eye_vs[0..t - 1],
                        &[],
                        Some(&light_sample.value),
                    );
                    (
                        light_emission * throughput * bsdf * g / light_sample.pdf,
                        mis_weight,
                    )
                }
            } else {
                let l_i = s - 2;
                if l_i >= len_l {
                    continue;
                }
                let v_light = &light_vs[l_i];
                let Vertex {
                    hit: e_hit,
                    throughput: e_throughput,
                    w_local: e_wout_local,
                    ..
                } = v_eye;
                let Vertex {
                    hit: l_hit,
                    throughput: l_throughput,
                    w_local: l_win_local,
                    ..
                } = v_light;
                if v_eye.hit.material.all_specular() {
                    continue;
                } else if v_light.hit.material.all_specular() {
                    continue;
                } else if !scene.visible(e_hit.pos(), l_hit.pos()) {
                    continue;
                }
                let e_to_l = (l_hit.pos() - e_hit.pos()).normalize();
                let e_win_local = e_hit.geom.lc().w2l() * e_to_l;
                let l_wout_local = l_hit.geom.lc().w2l() * -e_to_l;
                let g = e_hit.geom.g(l_hit.pos(), &l_hit.geom.gnorm);
                let l_bsdf = l_hit.material.bsdf(&l_win_local, &l_wout_local, false);
                let e_bsdf = e_hit.material.bsdf(&e_win_local, &e_wout_local, false);
                let mis_weight = mis_weight(
                    scene,
                    ray,
                    &eye_vs[0..t - 1],
                    &light_vs[0..s - 1],
                    Some(&light_sample.value),
                );
                let contrib =
                    *l_throughput * l_bsdf * g * e_bsdf * e_throughput * initial_ray.value.1
                        / initial_ray.pdf;
                (contrib, mis_weight)
            };

            accum_len += contrib * mis_weight;
        }

        radiance_accum.accum(&(accum_len, len - 2));
    }
}
