use serde::{Deserialize, Serialize};

/**
 * バックエンドで色々な処理することになったら別ライブラリとして分離したい
 */

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Figure {
    pub strokes: Vec<Stroke>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Stroke {
    pub points: Vec<Point>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Figure {
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn from_json(json: &str) -> Option<Figure> {
        serde_json::from_str(json).ok()
    }
}