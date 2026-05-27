# Static typing — annotations are checked at compile time (not CPython dynamic typing).

def main() -> int:
    a: str = "hello"
    n: int = 42
    print(a)
    print(n)
    return 0
