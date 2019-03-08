use crate::*;
use rgb::RGB;

#[derive(Clone, Debug)]
pub enum Material {
    Lambert(RGB),
}
