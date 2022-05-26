use serde::{Deserialize, Serialize};

/**
 * バックエンドで色々な処理することになったら別ライブラリとして分離したい
 */

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Figure {
    strokes: Vec<Stroke>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Stroke {
    points: Vec<Point>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Point {
    x: i32,
    y: i32,
    z: i32,
}

impl Figure {
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn from_json(json: &str) -> Figure {
        serde_json::from_str(json).unwrap()
    }
}
