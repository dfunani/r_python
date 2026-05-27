# Classes are the default way to combine state and behavior in rPython.

class Greeter:
    def greet(self) -> int:
        print("hello from Greeter")
        return 0

def main() -> int:
    Greeter().greet()
    return 0
