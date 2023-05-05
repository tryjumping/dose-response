#!/usr/bin/env python3

import json
import sys


with open(sys.argv[1], 'r') as i:
    for line in i:
        if len(line) > 0 and line[0] == '{':
            j = json.loads(line)
            del j["verification"]
            filtered_result = json.dumps(j)
        else:
            filtered_result = line.strip()
        print(filtered_result)
