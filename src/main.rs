use example_scenes;
use log::*;
use sabiptrace::*;

fn main() {
    use renderer::*;
    env_logger::init();
    use std::sync::{Arc, Mutex};

    let v = vec![RGB::all(0.0); 10];
    let film = {
        let s = 50;
        image::Film::new(16 * s, 9 * s, v.clone())
    };

    let (camera, scene) = example_scenes::make_box();
    let film = Arc::new(Mutex::new(film));
    let scene = Arc::new(scene);

    let film_config = FilmConfig {
        film_arc: film.clone(),
        accum_init: v.clone(),
    };

    let render_config = RenderConfig {
        integrator: Integrator::PathrTraceWithNee,
        nthread: num_cpus::get(),
    };

    const MAX_SPP: usize = 2000;
    const MAX_TIME_SEC: f64 = 20.0;
    const REPORT_FREQ: f64 = 5.0;
    let sched = {
        let start = std::time::Instant::now();
        let _film = film.clone();
        Box::new(move |next_cycle: usize, completed_samples: usize| {
            if next_cycle <= 0 {
                Some(1)
            } else {
                let elapsed = std::time::Instant::now().duration_since(start);
                let ms = { elapsed.as_secs() * 1000 + elapsed.subsec_millis() as u64 };
                let secs = (ms as f64) / 1000.0;
                let progress = completed_samples as f64 / MAX_SPP as f64;
                let eta = secs * (1.0 - progress) / progress;
                let spd = completed_samples as f64 / secs;
                let spd_pc = spd / render_config.nthread as f64;
                info!(
                    "completed {} / {} ({:.2} %) elapsed {:.2} sec ETA {:.2} sec ({:?} for limit)",
                    completed_samples,
                    MAX_SPP,
                    progress * 100.0,
                    secs,
                    eta,
                    MAX_TIME_SEC - secs
                );
                info!("Speed {:.2} spp/sec {:.2} spp/sec/core", spd, spd_pc);
                //let film = film.lock().unwrap();
                //film.to_image(RGBPixel::average).write_exr(&format!("output/{}.exr", cycle));
                if completed_samples >= MAX_SPP {
                    None
                } else if secs >= MAX_TIME_SEC {
                    info!("stopping due to time limit");
                    None
                } else {
                    let rest = MAX_SPP - completed_samples;
                    let next_cycle_time = REPORT_FREQ.min(MAX_TIME_SEC - secs);
                    let next_report: usize = (next_cycle_time * spd) as usize;
                    Some(rest.min(next_report).max(1))
                }
            }
        })
    };

    let renderer = Renderer;
    renderer.render(scene, &camera, film_config, render_config, sched);
    let film = film.lock().unwrap();
    for i in 0..v.len() {
        film.to_image(|v| v.accum[i] / v.samples as f32)
            .write_exr(&format!("output/len{:>02}.exr", i));
    }
    film.to_image(|v| {
        let mut sum = RGB::default();
        for c in v.accum.iter() {
            sum += *c;
        }
        sum / v.samples as f32
    })
    .write_exr("output/total.exr");
    //film.lock()
    //    .unwrap()
    //    .to_image(RGBPixel::average)
    //    .write_exr("output/output.exr");
}
