use serde::{Deserialize, Serialize};

use super::StrokeCount;

/**
 * バックエンドで色々な処理することになったら別ライブラリとして分離したい
 */

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Figure {
    strokes: Vec<Stroke>,
    width: f64,
    height: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Stroke {
    points: Vec<Point>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Point {
    x: f64,
    y: f64,
    z: f64,
}

impl Figure {
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn to_json_ast(&self) -> serde_json::Value {
        serde_json::to_value(&self).unwrap()
    }

    pub fn from_json(json: &str) -> Option<Figure> {
        Self::from_json_ast(serde_json::from_str(json).ok()?)
    }

    fn stroke_count_opt(&self) -> Option<StrokeCount> {
        StrokeCount::try_from(i32::try_from(self.strokes.len()).ok()?).ok()
    }

    pub fn from_json_ast(json: serde_json::Value) -> Option<Figure> {
        let figure: Figure = serde_json::from_value(json).ok()?;
        figure.stroke_count_opt()?;
        Some(figure)
    }

    pub fn stroke_count(&self) -> StrokeCount {
        self.stroke_count_opt().unwrap()
    }
}
