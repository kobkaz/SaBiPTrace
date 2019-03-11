use crate::*;
pub struct Image {
    w: u32,
    h: u32,
    buf: Vec<RGB>,
}

impl Image {
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
