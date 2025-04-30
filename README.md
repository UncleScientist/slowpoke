# slowpoke

A set of libraries for running turtle graphics in Rust

- slowpoke: the engine for calculating turtle graphics
- slowpoke-iced: a front end for slowpoke using the `iced` GUI crate
- slowpoke-ratatui: a front end for slowpoke using the `ratatui` TUI crate


## Drawing Layers

### Layer 1: Turtle Commands

Turtle commands are those that are issued by the user, e.g. "forward(40)" or
"right(90)". These are methods on the turtle struct that the user has.

```
    turtle.forward(40);
    turtle.right(90);
```

They are defined by the slowpoke::command::DrawRequest enum.

### Layer 2: Drawing Commands

The drawing commands are those that the `slowpoke` library generates in response
to the user's Turtle Commands. A `forward(40)` gets converted to a line segment
that has a start, end, color, width, and so on.

They are defined by the slowpoke::generate::DrawCommand enum.

### Layer 3: UI Commands

The drawing commands are passed to the User Interface, which then needs to
convert them into commands it can draw with.

All the UI libraries need to support the following turtle drawing commands:

* Draw a line
* Set line width
* Set line color
* Draw a polygon with the current line color
* Set fill color
* Fill a polygon (with or without drawing a perimeter in the current line color)

All the UI libraries need to support the following screen drawing commands:

* Set background color
* Set background image
* Getting and setting the screen size
* Send key and mouse events
* Generate timer ticks for updates
* Pop up input windows for reading numbers and text
* Return the current drawing image as a png/gif/bitmap
* Setting the window title

## Special Considerations

### Filled Polygons

To draw a filled polygon takes three steps from the user's point of view:

1. Start the polygon: `turtle.begin_fill()`
2. Draw the points using move and rotate commands, e.g.: `turtle.forward(...)`
3. Finish the polygon: `turtle.end_fill()`

Inside Layer 1, we have a list of all the commands, including begin & end of the
polygon. We don't actually "fill" the polygon until the `end_fill()` method is
called, BUT we can't draw the outline of the polygon until after we've drawn the
interior fill of the polygon. This means that, as we animate, we draw the lines
of the polygon, and then once the `end_fill()` method is called, we convert the
`begin_fill()` entry in the list of commands into a "fill a polygon", and then
draw the lines that make up the perimeter of the polygon afterwards. The
`end_fill()` command becomes a no-op.

| User's Commands     | Vec&lt;DrawCommand&gt;                                           |
| ------------------- | -----------------                                                |
| turtle.begin_poly() | [Filler]                                                         |
| turtle.forward(20); | [Filler, Line(a, b)]                                             |
| turtle.right(90);   | [Filler, Line(a, b)]                                             |
| turtle.forward(20); | [Filler, Line(a, b), Line(b, c)]                                 |
| turtle.right(90);   | [Filler, Line(a, b), Line(b, c)]                                 |
| turtle.forward(20); | [Filler, Line(a, b), Line(b, c), Line(c, d)]                     |
| turtle.end_poly();  | [DrawPolygon(points, color), Line(a, b), Line(b, c), Line(c, d)] |


This gets complicated by the fact that we need to be able to support `undo()`!
If the user creates a polygon, and then calls `turtle.undo()`, we need to go
from the `[DrawPolygon(points, color), ...]` back to `[Filler, ...]`.

