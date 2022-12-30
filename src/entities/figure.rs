use super::StrokeCount;

mod json_model {
    use serde::{Deserialize, Serialize};
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
}

#[derive(Clone, Debug)]
pub struct Figure {
    model: json_model::Figure,
    stroke_count: StrokeCount,
}

impl Figure {
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.model).unwrap()
    }

    pub fn to_json_ast(&self) -> serde_json::Value {
        serde_json::to_value(&self.model).unwrap()
    }

    pub fn from_json(json: &str) -> Option<Figure> {
        Self::from_json_ast(serde_json::from_str(json).ok()?)
    }

    pub fn from_json_ast(json: serde_json::Value) -> Option<Figure> {
        let model: json_model::Figure = serde_json::from_value(json).ok()?;
        let stroke_count = StrokeCount::try_from(i32::try_from(model.strokes.len()).ok()?).ok()?;
        Some(Figure {
            model,
            stroke_count,
        })
    }

    pub fn stroke_count(&self) -> StrokeCount {
        self.stroke_count
    }
}
