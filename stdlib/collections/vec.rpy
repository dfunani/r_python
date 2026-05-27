# Growable vector (v2 stdlib — backed by runtime when linked)

struct Vec[T]:
    data: int
    len: int
    cap: int
