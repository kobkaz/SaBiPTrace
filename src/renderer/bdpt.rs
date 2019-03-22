use super::*;
use scene::Scene;

#[allow(unused_variables)]
fn strategy_weight(s: usize, t: usize) -> Option<f32> {
    Some(1.0)
}

#[derive(Clone)]
struct Vertex {
    hit: object::ObjectHit,
    throughput: RGB,
    w_local: V3,
    pdf_area: f32,
    specular: bool,
}

impl Vertex {
    pub fn pos(&self) -> &P3 {
        &self.hit.geom.pos
    }

    pub fn gnorm(&self) -> &V3 {
        &self.hit.geom.gnorm
    }
}

#[derive(Clone)]
struct ExtVertex {
    pdf_area: f32,
    specular: bool,
}

impl ExtVertex {
    pub fn new(pdf_area: f32, specular: bool) -> Self {
        ExtVertex { pdf_area, specular }
    }
}

fn continue_chance_from_throughput(throughput: &RGB, depth: usize) -> f32 {
    let base = throughput.max().min(1.0).max(0.1);
    if depth < 3 {
        base
    } else {
        base * 0.5f32.powi((depth - 3) as i32)
    }
}

fn gen_vertices<R: ?Sized>(
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
        }

        let next = hit.material.sample_win_cos(&wout_local, rng);
        let win_local = next.value.0;

        vs.push(Vertex {
            hit: hit.clone(),
            throughput,
            w_local: wout_local,
            pdf_area,
            specular: next.value.2,
        });

        let bsdf_cos = next.value.1;
        throughput *= bsdf_cos / next.pdf;
        pdf_area *= next.pdf; // will be converted into area pdf in the next loop

        let cont = pdf::RandomBool {
            chance: continue_chance_from_throughput(&throughput, depth),
        };
        let cont = cont.sample(rng);
        if !cont.value {
            break;
        }

        throughput /= cont.pdf;
        pdf_area *= cont.pdf;
        ray = hit_lc.l2w() * Ray::new(P3::origin(), win_local);
    }
    vs
}

fn extend_path_pdf<'a>(
    ray_delta: bool,
    origin: &P3,
    vs_init: &'a [Vertex],
    vs_latter: &'a [Vertex],
    v_last: Option<(P3, V3, bool)>,
) -> impl Iterator<Item = ExtVertex> + 'a {
    use either::Either::{Left, Right};
    assert!(!vs_init.is_empty() || !vs_latter.is_empty());

    struct State {
        depth: usize,
        dir: V3,
        throughput: RGB,
        pdf_area: f32,
    };

    let (init, init_state) = {
        let init_pdf_area = vs_init.last().map(|v| v.pdf_area).unwrap_or_else(|| {
            if ray_delta {
                1.0
            } else {
                let v = &vs_latter.last().unwrap().hit.geom;
                let r = origin - v.pos;
                let r_norm = r.norm();
                let p = r.dot(&v.gnorm).abs() / r_norm / r_norm / r_norm;
                p
            }
        });
        let init_raydir = {
            let p = vs_init
                .last()
                .unwrap_or_else(|| &vs_latter.last().unwrap())
                .pos();
            let p_prev = if vs_init.len() <= 1 {
                origin
            } else {
                vs_init[vs_init.len() - 2].pos()
            };
            (p - p_prev).normalize()
        };
        let init = if vs_init.is_empty() {
            Left(
                vs_latter
                    .last()
                    .map(|v| ExtVertex::new(init_pdf_area, v.specular))
                    .into_iter(),
            )
        } else {
            Right(
                vs_init
                    .iter()
                    .map(|v| ExtVertex::new(v.pdf_area, v.specular)),
            )
        };
        let init_state = State {
            depth: if vs_init.len() == 0 {
                0
            } else {
                vs_init.len() - 1
            },
            dir: init_raydir,
            throughput: vs_init
                .last()
                .map(|v| v.throughput)
                .unwrap_or(RGB::all(1.0)),
            pdf_area: init_pdf_area,
        };
        (init, init_state)
    };

    let vs = vs_init.last().into_iter().chain(vs_latter.iter().rev());
    let next_vs = {
        let next_vs = vs_latter
            .iter()
            .map(|v| (*v.pos(), *v.gnorm(), v.specular))
            .rev()
            .chain(v_last);
        if vs_init.is_empty() {
            next_vs.skip(1)
        } else {
            next_vs.skip(0)
        }
    };

    assert!(init_state.throughput.max() >= 0.0);
    let latter = vs.zip(next_vs).scan(init_state, move |state, (v, next)| {
        let (next_pos, next_normal, next_specular) = next;
        let v_geom = &v.hit.geom;
        let lc = v_geom.lc();
        let wout_local = lc.w2l() * -state.dir;
        let to_next = next_pos - v_geom.pos;
        let r = to_next.norm();
        let win = to_next / r;
        let win_local = lc.w2l() * win;
        let dir_pdf_omega = v.hit.material.sample_win_pdf(&wout_local, &win_local);
        let bsdf_cos = v.hit.material.bsdf_cos(&win_local, &wout_local);
        assert!(bsdf_cos.max() >= 0.0);
        assert!(dir_pdf_omega >= 0.0);

        state.throughput *= bsdf_cos / dir_pdf_omega;
        state.pdf_area *= dir_pdf_omega;
        if !v.specular {
            state.pdf_area *= win.dot(&next_normal).abs() / r / r;
        }
        let cont_prob = continue_chance_from_throughput(&state.throughput, state.depth);
        state.throughput /= cont_prob;
        assert!(cont_prob >= 0.0);
        state.pdf_area *= cont_prob;
        state.dir = win;
        state.depth += 1;

        Some(ExtVertex {
            pdf_area: state.pdf_area,
            specular: next_specular,
        })
    });

    init.chain(latter)
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

    let eye_extended: Vec<_> = extend_path_pdf(
        true,
        &ray.origin,
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
            extend_path_pdf(false, &light_pos, &[], &eye_vs[0..eye_vs.len() - 1], None).collect()
        }
    } else if original_s == 1 {
        assert!(light_vs.is_empty());
        let light = light_sample.unwrap();
        light_pos_pdf *= scene.sample_light_pdf(&light.pos, light.obj_ix);
        light_dir_pdf *= 1.0; // TODO direction pdf
        extend_path_pdf(false, &light.pos, &[], &eye_vs, None).collect()
    } else {
        let light = light_sample.unwrap();
        light_pos_pdf *= scene.sample_light_pdf(&light.pos, light.obj_ix);
        light_dir_pdf *= 1.0; // TODO direction pdf
        extend_path_pdf(false, &light.pos, &light_vs, &eye_vs, None).collect()
    };

    let pdfs: Vec<f32> = (0..=original_s + original_t - 2)
        .map(|s| {
            let t = original_s + original_t - s;
            assert!(t >= 2);
            let c = strategy_weight(s, t);
            if c.is_none() {
                return 0.0;
            }
            let c = c.unwrap();
            let ExtVertex {
                pdf_area: eye_terminal_pdf,
                specular: eye_terminal_specular,
                ..
            } = eye_extended[t - 2];
            let pdf = if s == 0 {
                eye_terminal_pdf
            } else if s == 1 {
                if eye_terminal_specular {
                    0.0
                } else {
                    eye_terminal_pdf * light_pos_pdf
                }
            } else {
                let ExtVertex {
                    pdf_area: light_terminal_pdf,
                    specular: light_terminal_specular,
                    ..
                } = light_extended[s - 2];
                if eye_terminal_specular || light_terminal_specular {
                    0.0
                } else {
                    eye_terminal_pdf * light_terminal_pdf * light_pos_pdf * light_dir_pdf
                }
            };
            if !pdf.is_finite() || pdf < 0.0 {
                error!("negative or not finite pdf {}", pdf);
                0.0
            } else {
                c * pdf
            }
        })
        .collect();

    let sum = pdfs.iter().sum::<f32>();
    let original_pdf = pdfs[original_s];
    let weight = original_pdf / sum;
    if !weight.is_finite() {
        return 0.0;
    } else {
        weight
    }
}

pub fn radiance<R: ?Sized>(
    scene: &Scene,
    ray: &Ray,
    radiance_accum: &mut impl Accumulator<(RGB, usize)>,
    rng: &mut R,
) where
    R: Rng,
{
    const LE_MAX: usize = 20;
    const LL_MAX: usize = 20;
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
                if !scene.visible(light_pos, hit.pos()) {
                    (RGB::all(0.0), 0.0)
                } else {
                    let g = hit.geom.g(light_pos, light_normal);
                    let light_dir = (light_pos - hit.pos()).normalize();
                    let bsdf = hit.material.bsdf(&(hit_lc.w2l() * light_dir), &wout_local);
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
                if !scene.visible(e_hit.pos(), l_hit.pos()) {
                    continue;
                }
                let e_to_l = (l_hit.pos() - e_hit.pos()).normalize();
                let e_win_local = e_hit.geom.lc().w2l() * e_to_l;
                let l_wout_local = l_hit.geom.lc().w2l() * -e_to_l;
                let g = e_hit.geom.g(l_hit.pos(), &l_hit.geom.gnorm);
                let l_bsdf = l_hit.material.bsdf(&l_win_local, &l_wout_local);
                let e_bsdf = e_hit.material.bsdf(&e_win_local, &e_wout_local);
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
