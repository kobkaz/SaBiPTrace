use super::*;

pub fn radiance<R: ?Sized>(
    enable_nee: bool,
    scene: &Scene,
    ray: &Ray,
    radiance_accum: &mut impl Accumulator<(RGB, usize)>,
    rng: &mut R,
) where
    R: Rng,
{
    let mut ray = ray.clone();
    let mut throughput = RGB::all(1.0);
    let mut prev_specular = true;
    let mut last_ray_pdf = 1.0;

    const DEPTH_MAX: usize = 100;
    const MIS_PDF_WEIGHT_PT: f32 = 1.0;
    const MIS_PDF_WEIGHT_NEE: f32 = 1.0;
    for depth in 0..DEPTH_MAX {
        let hit = scene.test_hit(&ray, 1e-3, std::f32::MAX / 2.0);
        if let Some(hit) = hit {
            let hit_lc = hit.geom.lc();
            let wout_local = hit_lc.w2l() * -ray.dir;

            if let Some(emission) = hit.emission {
                if prev_specular || !enable_nee {
                    radiance_accum.accum((throughput * emission, depth));
                } else {
                    let pt_pdf_omega = last_ray_pdf;
                    let pt_pdf_area =
                        pt_pdf_omega * wout_local[2].abs() / hit.geom.dist / hit.geom.dist;
                    let nee_pdf_area = scene.sample_light_pdf(&hit.geom.pos, hit.obj_ix);
                    let mis_weight = MIS_PDF_WEIGHT_PT * pt_pdf_area
                        / (MIS_PDF_WEIGHT_PT * pt_pdf_area + MIS_PDF_WEIGHT_NEE * nee_pdf_area);
                    radiance_accum.accum((throughput * emission * mis_weight, depth));
                }
            }

            if enable_nee && !hit.material.all_specular() {
                if let Some(light_sample) = scene.sample_light(rng) {
                    let scene::LightSampleResult {
                        pos: ref light_pos,
                        normal: ref light_normal,
                        emission: light_emission,
                        ..
                    } = light_sample.value;
                    //dbg!(light_sample.pdf);
                    //dbg!(scene.sample_light_pdf(&light_pos, obj_ix));
                    if scene.visible(light_pos, &hit.geom.pos) {
                        let g = hit.geom.g(&light_pos, light_normal);
                        let light_dir = (light_pos - hit.geom.pos).normalize();
                        let win_local = hit_lc.w2l() * light_dir;
                        let bsdf = hit.material.bsdf(&win_local, &wout_local);
                        let nee_contrib = throughput * light_emission * bsdf * g / light_sample.pdf;
                        if !nee_contrib.is_finite() {
                            warn!("nee_radiance is not finite {:?}", nee_contrib);
                            warn!("> throughput {:?}", throughput);
                            warn!("> light_emission {:?}", light_emission);
                            warn!("> bsdf {:?}", bsdf);
                            warn!("> g {:?}", g);
                            warn!("> light_sample.pdf {:?}", light_sample.pdf);
                        } else {
                            let pt_pdf_omega = hit.material.sample_win_pdf(&wout_local, &win_local);
                            let pt_pdf_area = pt_pdf_omega * (light_normal.dot(&light_dir)).abs()
                                / hit.geom.dist
                                / hit.geom.dist;
                            let nee_pdf_area = light_sample.pdf;
                            let mis_weight = MIS_PDF_WEIGHT_NEE * nee_pdf_area
                                / (MIS_PDF_WEIGHT_PT * pt_pdf_area
                                    + MIS_PDF_WEIGHT_NEE * nee_pdf_area);
                            radiance_accum.accum((nee_contrib * mis_weight, depth + 1));
                        }
                    }
                }
            }

            last_ray_pdf = 1.0;
            let next = hit.material.sample_win_cos(&wout_local, rng);
            let win_local = next.value.0;
            let bsdf_cos = next.value.1;
            prev_specular = next.value.2;
            throughput *= bsdf_cos;
            throughput /= next.pdf;
            last_ray_pdf *= next.pdf;

            let cont = pdf::RandomBool {
                chance: (throughput.max() * 0.8).min(1.0).max(0.1),
            };

            let cont = cont.sample(rng);
            if !cont.value {
                break;
            }
            throughput /= cont.pdf;
            last_ray_pdf *= cont.pdf;

            if !throughput.is_finite() {
                warn!("throughput is not finite {:?}", throughput);
                warn!("> wout_local {:?}", wout_local);
                warn!("> hit.geom {:?}", hit.geom);
                warn!("> hit.material {:?}", hit.material);
                warn!("> next {:?}", next);
                warn!("> bsdf_cos {:?}", bsdf_cos);
                warn!("> next.pdf {:?}", next.pdf);
                warn!("> cont.pdf {:?}", cont.pdf);
                break;
            }

            ray = hit_lc.l2w() * Ray::new(P3::origin(), win_local);
        } else {
            radiance_accum.accum((scene.envmap_dir(&ray.dir) * throughput, depth));
            break;
        }
    }
}
