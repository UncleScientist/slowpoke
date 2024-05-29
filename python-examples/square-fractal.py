from turtle import *

setup(500, 500)

def square_fractal(order, length):
    if order == 0:
        forward(length)
        return

    if order == 3:
        print(f"pos = {pos()}");

    square_fractal(order - 1, length / 3);
    left(90)
    square_fractal(order - 1, length / 3);
    right(90)
    square_fractal(order - 1, length / 3);
    right(90)
    square_fractal(order - 1, length / 3);
    left(90)
    square_fractal(order - 1, length / 3);

tracer(False)
penup()
goto(-243/2, 243/2)
pendown()

for _ in range(4):
    square_fractal(4, 243)
    right(90)

tracer(True)

mainloop()
