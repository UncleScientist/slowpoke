use slowpoke::*;

fn main() {
    /*
    let mut cmds = vec![
        Command::PenUp,
        Command::Left(90.),
        Command::Forward(729. / 2.),
        Command::Right(180.),
        Command::PenDown,
    ];

    spiky_fractal(&mut cmds, 3, 729.);
    cmds.push(Command::GoTo(-729. / 2., 0.));
    square_fractal(&mut cmds, 3, 729.);
    cmds.push(Command::GoTo(0., 0.));
    */

    let mut turtle = Turtle::new();
    for _ in 0..4 {
        turtle.forward(40.);
        turtle.right(90.);
    }

    turtle.run();
}

/*

fn spiky_fractal(cmds: &mut Vec<Command>, order: usize, length: f64) {
    if order == 0 {
        cmds.push(Command::Forward(length));
    } else {
        spiky_fractal(cmds, order - 1, length / 3.);
        cmds.push(Command::Left(60.));
        spiky_fractal(cmds, order - 1, length / 3.);
        cmds.push(Command::Right(120.));
        spiky_fractal(cmds, order - 1, length / 3.);
        cmds.push(Command::Left(60.));
        spiky_fractal(cmds, order - 1, length / 3.);
    }
}

fn square_fractal(cmds: &mut Vec<Command>, order: usize, length: f64) {
    if order == 0 {
        cmds.push(Command::Forward(length));
    } else {
        square_fractal(cmds, order - 1, length / 3.);
        cmds.push(Command::Left(90.));
        square_fractal(cmds, order - 1, length / 3.);
        cmds.push(Command::Right(90.));
        square_fractal(cmds, order - 1, length / 3.);
        cmds.push(Command::Right(90.));
        square_fractal(cmds, order - 1, length / 3.);
        cmds.push(Command::Left(90.));
        square_fractal(cmds, order - 1, length / 3.);
    }
}
*/
