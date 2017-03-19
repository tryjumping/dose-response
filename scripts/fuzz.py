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


REQUEST_TIMEOUT = 3000


if __name__ == '__main__':
    context = zmq.Context()

    # Socket to talk to server
    print("Connecting to hello world server...")
    socket = context.socket(zmq.REQ)
    socket.connect("ipc:///tmp/dose-response.ipc")

    print "Connected to the server"
    commands = [random.choice(KEYS) for _ in range(100)]
    commands.append('Q')
    for command in commands:
        print "Sending command {}".format(command)
        read_list, write_list, error_list = zmq.select([socket], [socket], [socket])
        if write_list:
            message = json.dumps(key_from_code(command))
            write_list[0].send(message)
        else:
            print("ERROR: no writable sockets available")
            break


        if command == 'Q':
            break  # We're quitting, don't wait on a reply

        read_list, write_list, error_list = zmq.select([socket], [socket], [socket], timeout=3)
        if read_list:
            message = read_list[0].recv()
            # print("Received reply: {}".format(message))
            print("Received reply")
            time.sleep(0.3)
        else:
            print("ERROR: Timed out waiting for a response")
            break

    socket.close()
    context.term()
