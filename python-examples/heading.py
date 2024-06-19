import turtle

turtle.goto(90, 160)
turtle.dot()

turtle.goto(-50, 17)
h = turtle.towards(90, 160)

print(f"Turtle heading for {turtle.pos()} to (90, 160) = {h}")

turtle.setheading(h)
turtle.forward(200)

h = turtle.towards(0, 0)
print(f"Turtle heading for (0, 0) = {h}")
turtle.setheading(h)
turtle.forward(50)

turtle.mainloop()
