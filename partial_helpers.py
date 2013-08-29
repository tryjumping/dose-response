def inc(n):
    return n + 1

def dec(n):
    return n - 1

def add(n):
    return lambda increment: n + increment

def sub(n):
    return add(-n)

def bounded_add(lower_bound, n, upper_bound=None):
    if upper_bound is None:
        return lambda increment: max(n + increment, lower_bound)
    else:
        return lambda increment: min(max(n + increment, lower_bound),
                                     upper_bound)

def replace(n):
    return lambda _: n
