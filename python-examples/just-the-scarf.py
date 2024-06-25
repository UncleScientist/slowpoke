from turtle import *


# Doraemon with Python Turtle
def cssmartkids(x, y):
    penup()
    goto(x, y)
    pendown()

def muflar():
    fillcolor('#e70010')
    begin_fill()
    seth(0)
    fd(200)
    circle(-5, 90)
    fd(10)
    circle(-5, 90)
    fd(207)
    circle(-5, 90)
    fd(10)
    circle(-5, 90)
    end_fill()


def Doraemon():
    muflar()


if __name__ == '__main__':
    screensize(800, 600, "#f0f0f0")
    pensize(3)
    speed(9)
    Doraemon()
    mainloop()
