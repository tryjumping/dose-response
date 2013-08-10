from lib.enum import Enum


StateOfMind = Enum('StateOfMind', [
    'dead',
    'delirium_tremens',
    'severe_withdrawal',
    'withdrawal',
    'sober',
    'high',
    'very_high',
    'overdosed'
])

def enumerate_state_of_mind(som):
    """Return an enum representing the given state of mind."""
    if som <= 0:
        return StateOfMind.dead
    elif som <= 5:
        return StateOfMind.delirium_tremens
    elif som <= 25:
        return StateOfMind.severe_withdrawal
    elif som <= 50:
        return StateOfMind.withdrawal
    elif som <= 55:
        return StateOfMind.sober
    elif som <= 94:
        return StateOfMind.high
    elif som <= 99:
        return StateOfMind.very_high
    else:
        return StateOfMind.overdosed
