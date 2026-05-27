# Trait static dispatch demo — acceptance program for P8
# Status: NOT expected to compile until traits + monomorphization land.
# See docs/IMPLEMENTATION_STATUS.md

trait Show:
    def show(self) -> str:
        ...

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
