use std::collections::HashMap;

use crate::color_names::TurtleColor;

const CLASSIC: [[f32; 2]; 5] = [[0., 0.], [-15., 6.], [-10., 0.], [-15., -6.], [0., 0.]];
const ARROW: [[f32; 2]; 4] = [[0., 0.], [-10., 12.], [-10., -12.], [0., 0.]];
const CIRCLE: [[f32; 2]; 10] = [
    [0., 0.],
    [0., 3.],
    [-5., 9.],
    [-13., 9.],
    [-18., 3.],
    [-18., -3.],
    [-13., -9.],
    [-5., -9.],
    [0., -3.],
    [0., 0.],
];
const SQUARE: [[f32; 2]; 6] = [
    [0., 0.],
    [0., 8.],
    [-16., 8.],
    [-16., -8.],
    [0., -8.],
    [0., 0.],
];
const TRIANGLE: [[f32; 2]; 4] = [[0., 0.], [-16., 8.], [-16., -8.], [0., 0.]];
const TURTLE: [[f32; 2]; 24] = [
    [16., 0.],
    [14., -2.],
    [10., -1.],
    [7., -4.],
    [9., -7.],
    [8., -9.],
    [5., -6.],
    [1., -7.],
    [-3., -5.],
    [-6., -8.],
    [-8., -6.],
    [-5., -4.],
    [-7., 0.],
    [-5., 4.],
    [-8., 6.],
    [-6., 8.],
    [-3., 5.],
    [1., 7.],
    [5., 6.],
    [8., 9.],
    [9., 7.],
    [7., 4.],
    [10., 1.],
    [14., 2.],
];

// TODO: add complex turtle shape

const SHAPES: [(&str, &[[f32; 2]]); 6] = [
    ("classic", &CLASSIC),
    ("arrow", &ARROW),
    ("circle", &CIRCLE),
    ("square", &SQUARE),
    ("triangle", &TRIANGLE),
    ("turtle", &TURTLE),
];

// struct TurtleShapeComponent

#[derive(Debug, Clone)]
pub struct TurtleShape {
    pub name: String,
    pub poly: Vec<ShapeComponent>,
}

impl Default for TurtleShape {
    fn default() -> Self {
        let shape = ShapeComponent {
            polygon: PolygonPath::new(&CLASSIC), // TODO: maybe not allocate in default()?
            fill: TurtleColor::CurrentColor,
            outline: TurtleColor::CurrentColor,
        };
        Self {
            name: "classic".into(),
            poly: vec![shape],
        }
    }
}

impl TurtleShape {
    pub(crate) fn new(name: &str, polygon: PolygonPath) -> Self {
        let shape = ShapeComponent {
            polygon,
            fill: TurtleColor::CurrentColor,
            outline: TurtleColor::CurrentColor,
        };
        Self {
            name: name.into(),
            poly: vec![shape],
        }
    }

    pub(crate) fn multi(name: &str, poly: &[ShapeComponent]) -> Self {
        Self {
            name: name.into(),
            poly: poly.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TurtleShapeName {
    GetCurrent,
    Shape(String),
}

#[derive(Debug, Clone)]
pub struct ShapeComponent {
    pub polygon: PolygonPath,
    pub fill: TurtleColor,
    pub outline: TurtleColor,
}

#[derive(Debug, Clone)]
pub enum Shape {
    Polygon(ShapeComponent),
    Image(Vec<u8>),
    Compound(Vec<ShapeComponent>),
}

impl From<Vec<[f32; 2]>> for Shape {
    fn from(poly: Vec<[f32; 2]>) -> Self {
        Self::polygon(&poly)
    }
}

impl Shape {
    #[must_use]
    pub fn polygon(polygon: &[[f32; 2]]) -> Self {
        Self::Polygon(ShapeComponent {
            polygon: PolygonPath::new(polygon),
            fill: TurtleColor::CurrentColor,
            outline: TurtleColor::CurrentColor,
        })
    }

    #[must_use]
    pub fn compound() -> Self {
        Self::Compound(Vec::new())
    }

    pub fn addcomponent<F: Into<TurtleColor>, O: Into<TurtleColor>>(
        &mut self,
        polygon: &[[f32; 2]],
        fill: F,
        outline: O,
    ) {
        if let Shape::Compound(v) = self {
            v.push(ShapeComponent {
                polygon: PolygonPath::new(polygon),
                fill: fill.into(),
                outline: outline.into(),
            });
        };
    }
}

impl From<&str> for TurtleShapeName {
    fn from(name: &str) -> Self {
        Self::Shape(name.into())
    }
}

#[derive(Clone, Debug)]
pub struct PolygonPath {
    pub path: Vec<[f32; 2]>,
}

impl PolygonPath {
    pub fn new(diagram: &[[f32; 2]]) -> Self {
        Self {
            path: diagram.to_vec(),
        }
    }
}

pub(crate) fn generate_default_shapes() -> HashMap<String, TurtleShape> {
    let mut shapes = HashMap::new();

    for (name, poly) in &SHAPES {
        shapes.insert(
            (*name).into(),
            TurtleShape::new(name, PolygonPath::new(poly)),
        );
    }

    shapes
}
