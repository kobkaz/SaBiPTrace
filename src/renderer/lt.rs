use super::bdpt::gen_vertices;
use super::*;
use crate::*;
use camera::Camera;
use image::Image;
use scene::Scene;

pub fn sample<R, C, T>(
    scene: &Scene,
    camera: &C,
    film: &mut FilmVec<T>,
    _accum_init: &T,
    rng: &mut R,
) where
    R: Rng + ?Sized,
    C: Camera + ?Sized,
    T: Clone + Accumulator,
{
    let n_pixels = film.w() * film.h();
    let film_area = camera.film_width() * camera.film_width() * film.h() as f32 / film.w() as f32;
    let pixel_area = film_area / n_pixels as f32;

    //sample a light point from the scene
    let light_sample = if let Some(l) = scene.sample_light(rng) {
        l
    } else {
        return;
    };

    //generate path
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
    const MAX_DEPTH: usize = 25;
    let vs = gen_vertices(scene, &initial_ray.value.0, false, MAX_DEPTH, rng);

    for s in 1..=vs.len() + 1 {
        if s > 2 && vs[s - 2].specular {
            continue;
        }

        //position of vertex
        let pos = if s == 1 {
            &light_sample.value.pos
        } else {
            vs[s - 2].pos()
        };

        let film_sample = camera.sample_film_uv(pos, rng);
        //skip if the vertex is not visible from the camera
        if film_sample.is_none() {
            continue;
        }
        let film_sample = film_sample.unwrap();
        //get corresponding pixel's index
        let (ix_x, ix_y) = {
            let u = film_sample.u;
            let v = film_sample.v;
            let ixs = film.uv_to_ix_in_range(u, v);
            if ixs.is_none() {
                continue;
            }
            ixs.unwrap()
        };
        //skip if the vertex is occluded
        if !scene.visible(&film_sample.lens_point, pos) {
            continue;
        }

        let (radiance, cos) = if s == 1 {
            let cos = (light_sample.value.pos - film_sample.lens_point)
                .normalize()
                .dot(&light_sample.value.normal)
                .abs();
            (light_sample.value.emission / light_sample.pdf, cos)
        } else {
            let vertex = &vs[s - 2];

            let win_local = vertex.w_local;
            let wout = (film_sample.lens_point - vertex.pos()).normalize();
            let wout_local = vertex.hit.geom.lc().w2l() * wout;
            let bsdf = vertex.hit.material.bsdf(&win_local, &wout_local, false);

            let radiance = initial_ray.value.1 / initial_ray.pdf * vertex.throughput * bsdf;
            (radiance, wout_local[2].abs())
        };

        //dA(x_film) = measure_conv * cos * dA(x)
        let contrib = radiance * film_sample.measure_conv * cos / pixel_area;
        film.at_mut(ix_x as usize, ix_y as usize)
            .accum(&(contrib, s - 1));
    }
}
