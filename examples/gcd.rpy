# Greatest common divisor — target example for P4+ (control flow + calls)
# Status: may require tuple assignment / modulo in MIR; see IMPLEMENTATION_STATUS.md

def gcd(a: int, b: int) -> int:
    while b != 0:
        t = a % b
        a = b
        b = t
    return a

def main() -> int:
    print(gcd(48, 18))
    return 0
