# Interface static dispatch demo — target for P8 (v2)
# Prefer `interface` over deprecated `trait`.

interface Show:
    def show(self) -> str

struct Point:
    x: int
    y: int

impl Show for Point:
    def show(self) -> str:
        return "Point"

def main() -> int:
    p = Point { x: 1, y: 2 }
    print(p.show())
    return 0
