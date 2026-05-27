def fizzbuzz(n: int) -> int:
    i = 1
    while i <= n:
        if i % 15 == 0:
            print("FizzBuzz")
        elif i % 3 == 0:
            print("Fizz")
        elif i % 5 == 0:
            print("Buzz")
        else:
            print(i)
        i = i + 1
    return 0

def main() -> int:
    fizzbuzz(15)
    return 0
