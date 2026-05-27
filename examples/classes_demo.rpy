# Classes are the default way to combine state and behavior in rPython.
# Use `struct` only for plain data (C-compatible layout) without methods.

class Greeter:
    message: str

    def greet(self) -> int:
        print(self.message)
        return 0

def main() -> int:
    g = Greeter()
    return g.greet()
