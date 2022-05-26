/**
 * バックエンドで色々な処理することになったら別ライブラリとして分離したい
 */

pub struct Figure {
    strokes: Vec<Stroke>,
}

pub struct Stroke {
    points: Vec<Point>,
}

pub struct Point {
    x: i32,
    y: i32,
    z: i32,
}
