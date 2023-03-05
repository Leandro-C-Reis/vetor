use crate::BoneType;

#[derive(Debug, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub typo: u8,
    pub parent: usize,
    pub index: usize,
    pub chidren: Vec<usize>
}