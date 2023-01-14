use crate::accum::*;
use crate::*;
use log::*;
use rand::prelude::*;
use std::ops::{Deref, DerefMut};
use std::sync::*;


pub struct Image {
    w: usize,
    h: usize,
    buf: Vec<RGB>,
}

impl Image {
    pub fn read_exr16(file: &str) -> Option<Self> {
        /*
        use openexr::*;
        let mut file = std::fs::File::open(file).ok()?;
        let mut file = InputFile::new(&mut file).ok()?;
        let (w, h) = file.header().data_dimensions();
        let w = w as usize;
        let h = h as usize;
        //let mut buf: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; (w * h) as usize];
        let mut buf: Vec<RGB16> = vec![Default::default(); (w * h) as usize];
        {
            let mut fb = FrameBufferMut::new(w as u32, h as u32);
            fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut buf);
            file.read_pixels(&mut fb).ok();
        }
        Some(Image {
            w,
            h,
            buf: buf.into_iter().map(Into::into).collect(),
        })
        */
        unimplemented!()
    }

    pub fn new(w: usize, h: usize) -> Self {
        let mut buf = Vec::new();
        buf.resize((w * h) as usize, RGB::new(0.0, 0.0, 0.0));
        Image { w, h, buf }
    }

    pub fn write_exr(&self, filename: &str) {
        exr::prelude::write_rgb_file(filename, self.w, self.h, |x, y| {
            let pixel = self.at(x, y);
            (pixel.r, pixel.g, pixel.b)
        }).unwrap();
    }

    pub fn at_uv(&self, u: f32, v: f32) -> &RGB {
        let w = self.w as f32;
        let h = self.h as f32;
        let x = (w * (u + 1.0) / 2.0) as i32;
        let x = (x.max(0) as usize).min(self.w - 1);
        let y = (h * (1.0 - v) / 2.0) as i32;
        let y = (y.max(0) as usize).min(self.h - 1);
        self.at(x, y)
    }

    pub fn at(&self, x: usize, y: usize) -> &RGB {
        &self.buf[(y * self.w + x) as usize]
    }

    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut RGB {
        &mut self.buf[(y * self.w + x) as usize]
    }

    pub fn w(&self) -> usize {
        self.w
    }
    pub fn h(&self) -> usize {
        self.h
    }
}

#[derive(Clone)]
pub struct Film<B> {
    w: usize,
    h: usize,
    buf: B,
}
impl<B> Film<B> {
    pub fn w(&self) -> usize {
        self.w
    }
    pub fn h(&self) -> usize {
        self.h
    }
}

pub type FilmArc<T> = Film<Arc<Mutex<Vec<Pixel<T>>>>>;
pub type FilmVec<T> = Film<Vec<Pixel<T>>>;
impl<B> Film<B> {
    pub fn sample_uv_in_pixel(
        &self,
        xi: i32,
        yi: i32,
        rng: &mut (impl Rng + ?Sized),
    ) -> (f32, f32) {
        use rand::distributions::Uniform;
        let u = {
            let x = xi as f32 + Uniform::new(0.0, 1.0).sample(rng);
            x / self.w() as f32 - 0.5
        };
        let v = {
            let y = yi as f32 + Uniform::new(0.0, 1.0).sample(rng);
            (self.h() as f32 / 2.0 - y) / self.w() as f32
        };
        (u, v)
    }

    pub fn uv_to_ix(&self, u: f32, v: f32) -> (i32, i32) {
        let x = (u + 0.5) * self.w() as f32;
        let y = self.h() as f32 / 2.0 - v * self.w() as f32;
        (x as i32, y as i32)
    }
    pub fn uv_to_ix_in_range(&self, u: f32, v: f32) -> Option<(usize, usize)> {
        let (xi, yi) = self.uv_to_ix(u, v);
        if xi < 0 || self.w() <= xi as usize {
            None
        } else if yi < 0 || self.h() <= yi as usize {
            None
        } else {
            Some((xi as usize, yi as usize))
        }
    }
}

impl<T: Clone> FilmVec<T> {
    pub fn new(w: usize, h: usize, v: T) -> Self {
        let mut buf = Vec::new();
        buf.resize(
            (w * h) as usize,
            Pixel {
                accum: v,
                samples: 0,
            },
        );
        FilmVec { w, h, buf }
    }

    pub fn into_arc(self) -> FilmArc<T> {
        Film {
            w: self.w,
            h: self.h,
            buf: Arc::new(Mutex::new(self.buf)),
        }
    }
}

impl<T: Accumulator> FilmVec<T> {
    pub fn reset(&mut self) {
        for pixel in self.buf.iter_mut() {
            pixel.reset()
        }
    }
}

impl<T, B: Deref<Target = [Pixel<T>]>> Film<B> {
    pub fn to_image(&self, f: impl FnMut(&Pixel<T>) -> RGB) -> Image {
        Image {
            w: self.w,
            h: self.h,
            buf: self.buf.iter().map(f).collect(),
        }
    }
}

impl<T, B: DerefMut<Target = [Pixel<T>]>> Film<B> {
    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut Pixel<T> {
        &mut self.buf.deref_mut()[(y * self.w + x) as usize]
    }
}

impl<T> FilmArc<T> {
    pub fn with_lock<F, A>(&self, f: F) -> Result<A, PoisonError<MutexGuard<Vec<Pixel<T>>>>>
    where
        F: FnOnce(Film<&mut [Pixel<T>]>) -> A,
    {
        self.buf.lock().map(|mut mg| {
            f(Film {
                w: self.w,
                h: self.h,
                buf: &mut **mg,
            })
        })
    }
}
