//use log::*;
use sabiptrace::*;
use example_scenes;

fn main() {
    use renderer::*;
    use image::RGBPixel;
    env_logger::init();
    use std::sync::{Arc, Mutex};

    let v = vec![RGB::all(0.0); 10];
    let film = {
        let s = 50;
        image::Film::new(16 * s, 9 * s, v.clone())
    };

    let (camera, scene) = example_scenes::make_debug();
    let film = Arc::new(Mutex::new(film));
    let scene = Arc::new(scene);

    let film_config = FilmConfig {
        film_arc: film.clone(),
        accum_init: v.clone(),
    };

    let render_config = RenderConfig {
        integrator: Integrator::BidirectionalPathTrace,
        nthread: num_cpus::get(),
        spp: 1000,
        cycle_spp: 100,
    };

    let cb = {
        let start = std::time::Instant::now();
        //let film = film.clone();
        let mut cycle = 0;
        Box::new(move |completed_samples, total_samples| {
            cycle += 1;
            let elapsed = std::time::Instant::now().duration_since(start);
            let ms = { elapsed.as_secs() * 1000 + elapsed.subsec_millis() as u64 };
            let secs = (ms as f64) / 1000.0;
            let progress = completed_samples as f64 / total_samples as f64;
            let eta = secs * (1.0 - progress) / progress;
            let spd = completed_samples as f64 / secs;
            let spd_pc = spd / render_config.nthread as f64;
            println!(
                "completed {} / {} ({:.2} %) elapsed {:.2} sec  ETA {:.2} sec",
                completed_samples,
                total_samples,
                progress * 100.0,
                secs,
                eta
                );
            println!("Speed {:.2} spp/sec {:.2} spp/sec/core", spd, spd_pc);
            //let film = film.lock().unwrap();
            //film.to_image(RGBPixel::average).write_exr(&format!("output/{}.exr", cycle));
        })
    };

    let renderer = Renderer;
    renderer.render(scene, &camera, film_config, render_config, cb);
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
