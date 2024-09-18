use std::collections::HashMap;

use iced::widget::canvas::Path;

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

// TODO: add complex turtle shape

const SHAPES: [(&str, &[[f32; 2]]); 5] = [
    ("classic", &CLASSIC),
    ("arrow", &ARROW),
    ("circle", &CIRCLE),
    ("square", &SQUARE),
    ("triangle", &TRIANGLE),
];

#[derive(Debug, Clone)]
pub struct TurtleShape {
    pub(crate) name: String,         // TODO: make accessor
    pub(crate) shape: TurtlePolygon, // TODO: make a get_path()
}

impl Default for TurtleShape {
    fn default() -> Self {
        Self {
            name: "classic".into(),
            shape: TurtlePolygon::new(&CLASSIC),
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
    // TODO: can ShapeComponent be "more private" than Shape?
    pub(crate) polygon: TurtlePolygon,
    pub(crate) fill: TurtleColor,
    pub(crate) outline: TurtleColor,
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
    pub fn polygon(polygon: &[[f32; 2]]) -> Self {
        Self::Polygon(ShapeComponent {
            polygon: TurtlePolygon::new(polygon),
            fill: TurtleColor::CurrentColor,
            outline: TurtleColor::CurrentColor,
        })
    }

    pub fn addcomponent<F: Into<TurtleColor>, O: Into<TurtleColor>>(
        &mut self,
        polygon: &[[f32; 2]],
        fill: F,
        outline: O,
    ) {
        if let Shape::Compound(v) = self {
            v.push(ShapeComponent {
                polygon: TurtlePolygon::new(polygon),
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
pub struct TurtlePolygon {
    path: Path,
}

impl TurtlePolygon {
    pub fn new(diagram: &[[f32; 2]]) -> Self {
        let mut iter = diagram.iter();

        let first = iter.next().unwrap();
        let path = Path::new(|b| {
            b.move_to((*first).into());
            for i in iter {
                b.line_to((*i).into());
            }
        });

        Self { path }
    }

    pub(crate) fn get_path(&self) -> &Path {
        &self.path
    }
}

pub(crate) fn generate_default_shapes() -> HashMap<String, TurtleShape> {
    let mut shapes = HashMap::new();

    for (name, poly) in &SHAPES {
        shapes.insert(
            (*name).into(),
            TurtleShape {
                name: (*name).into(),
                shape: TurtlePolygon::new(poly),
            },
        );
    }

    shapes
}
