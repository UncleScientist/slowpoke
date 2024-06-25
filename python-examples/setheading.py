import turtle
from time import sleep

turtle.speed(1)
turtle.dot(3, "red")
sleep(10)

turtle.penup()
turtle.goto(10, 20)
turtle.pendown()

# draw to the right
turtle.setheading(0)
turtle.forward(100)
print(f"pos = {turtle.pos()}")

turtle.penup()
turtle.goto(10, 20)
turtle.pendown()

# draw "south" or down
turtle.setheading(270)
turtle.forward(100)
print(f"pos = {turtle.pos}")

turtle.penup()
turtle.goto(10, 20)
turtle.pendown()

# draw "north" or up
turtle.setheading(90)
turtle.forward(100)
print(f"pos = {turtle.pos}")

turtle.penup()
turtle.goto(10, 20)
turtle.pendown()

# draw "west" or to the left
turtle.setheading(180)
turtle.forward(100)
print(f"pos = {turtle.pos}")

turtle.penup()
turtle.goto(10, 20)
turtle.pendown()

turtle.left(30)

turtle.mainloop()
