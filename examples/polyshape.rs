use slowpoke::{Shape, SlowpokeLib};

fn main() {
    SlowpokeLib::default()
        .with_size(400, 400)
        .with_title("polygon test")
        .run(|turtle| {
            let mut compound = Shape::compound();
            compound.addcomponent(
                &[
                    [50., -20.],
                    [30., 20.],
                    [-50., 20.],
                    [-30., -20.],
                    [50., -20.],
                ],
                "red",
                "blue",
            );
            compound.addcomponent(
                &[
                    [-16., 0.],
                    [-14., -2.],
                    [-10., -1.],
                    [-7., -4.],
                    [-9., -7.],
                    [-8., -9.],
                    [-5., -6.],
                    [-1., -7.],
                    [3., -5.],
                    [6., -8.],
                    [8., -6.],
                    [5., -4.],
                    [7., 0.],
                    [5., 4.],
                    [8., 6.],
                    [6., 8.],
                    [3., 5.],
                    [-1., 7.],
                    [-5., 6.],
                    [-8., 9.],
                    [-9., 7.],
                    [-7., 4.],
                    [-10., 1.],
                    [-14., 2.],
                ],
                "green",
                "light green",
            );
            turtle.register_shape("pg", compound);
            turtle.shape("pg");

            turtle.fd(100);
            turtle.rt(90);
            turtle.fd(100);
        });
}
