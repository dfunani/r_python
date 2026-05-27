struct Point:
    x: int
    y: int

def main() -> int:
    p = Point { x: 3, y: 4 }
    print(p.x)
    print(p.y)
    return 0
