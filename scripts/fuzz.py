#!/usr/bin/env python


import json
import random
import subprocess
import sys
import time
import zmq


KEYS = [
    'Up', 'Down', 'Left', 'Right',
    'NumPad7', 'NumPad9', 'NumPad1', 'NumPad3',
    'D1', 'D2', 'D3', 'D4', 'D5',
]


def key_from_code(key_code):
    return {
        "code": key_code,
        "alt": False,
        "ctrl": False,
        "shift": False,
    }


def surroundings_from_message(message):
    width = message['width']
    height = message['height']

    def get_cell(x, y):
        return message['cells'][y * width + x]

    def cell_type(cell):
        types = {
            '@': 'player',
            '.': 'empty',
            '#': 'wall',
            'a': 'monster',
            'h': 'monster',
            'D': 'monster',
            'S': 'monster',
            'v': 'monster',
            '%': 'food',
            'i': 'dose',
            'I': 'dose',
            '+': 'dose',
            'x': 'dose',
        }
        if cell in types:
            return types[cell]
        else:
            return 'unknown'

    assert len(message['cells']) == width * height
    for x in range(width):
        for y in range(height):
            if cell_type(get_cell(x, y)) == 'player':
                player_x = x
                player_y = y

    x, y = player_x, player_y
    result = {
        'NW': cell_type(get_cell(x - 1, y - 1)),
        'N':  cell_type(get_cell(x, y - 1)),
        'NE': cell_type(get_cell(x + 1, y - 1)),

        'W': cell_type(get_cell(x - 1, y)),
        'E': cell_type(get_cell(x + 1, y)),

        'SW': cell_type(get_cell(x - 1, y + 1)),
        'S':  cell_type(get_cell(x, y + 1)),
        'SE': cell_type(get_cell(x + 1, y + 1))
    }
    return result


def next_command(previous_command, display):
    all_directions = 'NW N NE W E SW S SE'.split()
    if not display or not previous_command:
        return random.choice(all_directions)
    if random.random() <= 0.07:
        return 'Quit'

    # Always go for food
    food_directions = [direction for direction, tile in display.items()
                       if tile == 'food']
    if food_directions:
        return random.choice(food_directions)

    dose_directions = [direction for direction, tile in display.items()
                       if tile == 'dose']
    if dose_directions:
        return random.choice(dose_directions)

    walkable_directions = [direction for direction, tile in display.items()
                           if tile in ('empty', 'monster')]
    if walkable_directions:
        return random.choice(walkable_directions)

    return random.choice(all_directions)


def key_from_command(command):
    mapping = {
        'NW': 'NumPad7',
        'N':  'Up',
        'NE': 'NumPad9',
        'W':  'Left',
        'E':  'Right',
        'SW': 'NumPad1',
        'S':  'Down',
        'SE': 'NumPad3',
        'Eat': 'D1',
        'Quit': 'Q',
    }
    return key_from_code(mapping[command])


# Run the game server:
# cargo run --features="remote opengl" -- --exit-after --invincible --replay-file ~/tmp/replay.txt --remote
# Replay:
# cargo run --features="remote opengl" -- --invincible ~/tmp/replay.txt
# Headless replay:
# cargo run --features="remote opengl" -- --invincible ~/tmp/replay.txt --exit-after --remote

if __name__ == '__main__':
    context = zmq.Context()

    # Socket to talk to server
    print("Connecting to hello world server...")
    socket = context.socket(zmq.REQ)
    socket.connect("ipc:///tmp/dose-response.ipc")

    print "Connected to the server"

    previous_command = None
    display = None

    while True:
        command = next_command(previous_command, display)
        key = key_from_command(command)
        previous_command = command

        print "Sending command: {}, key: {}".format(command, key)
        read_list, write_list, error_list = zmq.select([socket], [socket], [socket])
        if write_list:
            message = json.dumps(key)
            write_list[0].send(message)
        else:
            print("ERROR: no writable sockets available")
            break

        if command == 'Quit':
            break  # We're quitting, don't wait on a reply

        read_list, write_list, error_list = zmq.select([socket], [socket], [socket], timeout=3)
        if read_list:
            message = read_list[0].recv()
            # print("Received reply: {}".format(message))
            print("Received reply")
            display = surroundings_from_message(json.loads(message))
            print display
            time.sleep(0.3)
        else:
            print("ERROR: Timed out waiting for a response")
            break

    socket.close()
    context.term()
