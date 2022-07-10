use serde::{Deserialize, Serialize};

/**
 * バックエンドで色々な処理することになったら別ライブラリとして分離したい
 */

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Figure {
    pub strokes: Vec<Stroke>,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Stroke {
    pub points: Vec<Point>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Figure {
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn to_json_ast(&self) -> serde_json::Value {
        serde_json::to_value(&self).unwrap()
    }

    pub fn from_json(json: &str) -> Option<Figure> {
        serde_json::from_str(json).ok()
    }

    pub fn from_json_ast(json: serde_json::Value) -> Option<Figure> {
        serde_json::from_value(json).ok()
    }
}
