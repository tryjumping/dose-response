#!/usr/bin/env python


import datetime
import json
import os
import random
import shutil
import subprocess
import sys
import tempfile
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
    directions = 'NW N NE W E SW S SE'.split()
    if not display or not previous_command:
        return random.choice(directions)

    # Always go for food
    food_directions = [direction for direction, tile in display.items()
                       if tile == 'food']
    if food_directions:
        return random.choice(food_directions)

    dose_directions = [direction for direction, tile in display.items()
                       if tile == 'dose']
    if dose_directions:
        return random.choice(dose_directions)

    # Try to go in the previous direction
    if previous_command in directions and random.random() <= 0.75:
        if display[previous_command] in ('empty', 'monster'):
            return previous_command
        adjacent_directions = {
            'NW': ['N', 'W'],
            'N':  ['NE', 'NW'],
            'NE': ['N', 'E'],
            'W':  ['NW', 'SW'],
            'E':  ['NE', 'SE'],
            'SW': ['S', 'W'],
            'S':  ['SE', 'SW'],
            'SE': ['S', 'E'],
        }
        directions = [direction for direction in adjacent_directions
                      if display[direction] in ('empty', 'monster')]
        if directions:
            return random.choice(directions)

    walkable_directions = [direction for direction, tile in display.items()
                           if tile in ('empty', 'monster')]
    if walkable_directions:
        return random.choice(walkable_directions)

    return random.choice(directions)


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


def run_game():
    context = zmq.Context()

    # Socket to talk to server
    print("Connecting to hello world server...")
    socket = context.socket(zmq.REQ)
    socket.connect("ipc:///tmp/dose-response.ipc")

    print "Connected to the server"

    turns = 0
    max_turns = 200 + random.randint(10, 200)

    previous_command = None
    display = None

    while True:
        turns += 1

        if turns > max_turns:
            command = 'Quit'
        else:
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
            display = surroundings_from_message(json.loads(message))
            time.sleep(0.3)
        else:
            print("ERROR: Timed out waiting for a response")
            break

    socket.close()
    context.term()


def test_run():
    replay_file = tempfile.NamedTemporaryFile(delete=False)
    replay_file.close()  # We won't write anything, the game will
    print "Running the game with a replay destination: {}".format(
        replay_file.name)
    game_command = ['cargo', 'run', '--features=remote', '--',
                    '--remote', '--exit-after', '--invincible',
                    '--replay-file', replay_file.name]
    game = subprocess.Popen(game_command,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    time.sleep(1)

    if game.poll() is None:
        print "Sending commands to the game"
        run_game()
    else:
        print "ERROR: game ended prematurely."
        print "stdout: {}".format(game.stdout.read())
        print "stderr: {}".format(game.stderr.read())
        return

    print "The game ended, getting its status"
    time.sleep(1)
    rc = game.poll()
    if rc is None:
        game.kill()  # The game was still running, kill it
    elif rc == 0:
        print "Starting the replay"
    else:
        print "Dose response finished with return code: {}".format(rc)
        print "stdout: {}".format(game.stdout.read())
        print "stderr: {}".format(game.stderr.read())
        return

    replay_command = ['cargo', 'run', '--features=remote', '--',
                      '--remote', '--exit-after', '--invincible',
                      replay_file.name]
    game = subprocess.Popen(replay_command,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    rc = game.wait()
    if rc == 0:
        print "Replay finished successfully. No need to store it."
    else:
        # We got a bug / replay failure
        target_dir = os.path.join(os.curdir, 'replays', 'bugs')
        now = datetime.datetime.now()
        bug_path = os.path.join(target_dir, now.strftime('%Y-%m-%dT%H-%M-%S'))
        print "Recording crash to: {}".format(bug_path)
        if not os.path.isdir(target_dir):
            os.mkdir(target_dir)
        shutil.copyfile(replay_file.name, bug_path)
    os.unlink(replay_file.name)


if __name__ == '__main__':
    test_run()
