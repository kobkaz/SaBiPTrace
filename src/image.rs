use crate::*;
pub struct Image {
    w: u32,
    h: u32,
    buf: Vec<RGB>,
}

impl Image {
    pub fn read_exr16(file: &str) -> Option<Self> {
        use openexr::*;
        let mut file = std::fs::File::open(file).ok()?;
        let mut file = InputFile::new(&mut file).ok()?;
        let (w, h) = file.header().data_dimensions();
        //let mut buf: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; (w * h) as usize];
        let mut buf: Vec<RGB16> = vec![Default::default(); (w * h) as usize];
        {
            let mut fb = FrameBufferMut::new(w, h);
            fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut buf);
            file.read_pixels(&mut fb).ok();
        }
        Some(Image {
            w,
            h,
            buf: buf.into_iter().map(Into::into).collect(),
        })
    }

    pub fn new(w: u32, h: u32) -> Self {
        let mut buf = Vec::new();
        buf.resize((w * h) as usize, RGB::new(0.0, 0.0, 0.0));
        Image { w, h, buf }
    }

    pub fn write_exr(&self, filename: &str) {
        use openexr::*;

        let mut file = std::fs::File::create(filename).unwrap();
        let mut file = ScanlineOutputFile::new(
            &mut file,
            Header::new()
                .set_resolution(self.w, self.h)
                .add_channel("R", PixelType::FLOAT)
                .add_channel("G", PixelType::FLOAT)
                .add_channel("B", PixelType::FLOAT),
        )
        .unwrap();
        let mut buffer = FrameBuffer::new(self.w, self.h);
        buffer.insert_channels(&["R", "G", "B"], &self.buf[..]);
        file.write_pixels(&buffer).unwrap();
    }

    pub fn at_uv(&self, u: f32, v: f32) -> &RGB {
        let w = self.w as f32;
        let h = self.h as f32;
        let x = (w * (u + 1.0) / 2.0) as i32;
        let x = (x.max(0) as u32).min(self.w - 1);
        let y = (h * (1.0 - v) / 2.0) as i32;
        let y = (y.max(0) as u32).min(self.h - 1);
        self.at(x, y)
    }

    pub fn at(&self, x: u32, y: u32) -> &RGB {
        &self.buf[(y * self.w + x) as usize]
    }

    pub fn at_mut(&mut self, x: u32, y: u32) -> &mut RGB {
        &mut self.buf[(y * self.w + x) as usize]
    }

    pub fn w(&self) -> u32 {
        self.w
    }
    pub fn h(&self) -> u32 {
        self.h
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Pixel {
    pub accum: RGB,
    pub samples: usize,
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            accum: RGB::all(0.0),
            samples: 0,
        }
    }
}

pub struct Film {
    w: u32,
    h: u32,
    buf: Vec<Pixel>,
}

impl Film {
    pub fn new(w: u32, h: u32) -> Self {
        let mut buf = Vec::new();
        buf.resize((w * h) as usize, Default::default());
        Film { w, h, buf }
    }

    pub fn to_image(&self) -> Image {
        Image {
            w: self.w,
            h: self.h,
            buf: self
                .buf
                .iter()
                .map(|p| p.accum / (p.samples as f32))
                .collect(),
        }
    }

    pub fn at_mut(&mut self, x: u32, y: u32) -> &mut Pixel {
        &mut self.buf[(y * self.w + x) as usize]
    }

    pub fn w(&self) -> u32 {
        self.w
    }
    pub fn h(&self) -> u32 {
        self.h
    }
}
