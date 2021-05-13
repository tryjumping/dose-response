#!/usr/bin/env python3

from glob import glob
import os
import platform
import shutil
from sys import argv


def mkdir_p(directory):
    try:
        os.makedirs(directory)
    except OSError:
        pass
    return directory


if __name__ == '__main__':
    print(repr(argv))
    target_triple = argv[1]
    tag = argv[2]
    commit_hash = argv[3]

    out_path = os.path.join("target", "publish", "Dose Response")
    mkdir_p(out_path)

    system = platform.system()
    exe_name = 'dose-response'
    exe_extension = ''
    debug_script = 'debug.sh'
    if system == 'Windows':
        exe_extension = '.exe'
        debug_script = 'Debug.bat'
    full_exe_name = exe_name + exe_extension
    shutil.copy(os.path.join("target", "release", full_exe_name), out_path)

    # NOTE: this converts the line endings into the current platform's expected format:
    with open("README.md", 'r') as source:
        with open(os.path.join(out_path, 'README.txt'), 'w') as destination:
            destination.writelines(source.readlines())
    with open("COPYING.txt", 'r') as source:
        with open(os.path.join(out_path, 'LICENSE.txt'), 'w') as destination:
            destination.writelines(source.readlines())
    shutil.copy(debug_script, out_path)

    version_contents = f"Version: {tag}\nFull Version: dose-response-{tag}-{target_triple}\nCommit: {commit_hash}\n"
    with open(os.path.join(out_path, 'VERSION.txt'), 'w') as f:
        f.write(version_contents)

    print("Adding icons...")
    icons_destination_path = os.path.join(out_path, 'icons')
    mkdir_p(icons_destination_path)
    for filename in glob('assets/icon*'):
        shutil.copy(filename, icons_destination_path)

    archive_extension = 'tar.gz'
    if system in ('Windows', 'Darwin'):
        archive_extension = 'zip'

    if archive_extension == 'zip':
        archive_format = 'zip'
    elif archive_extension == 'tar.gz':
        archive_format = 'gztar'
    else:
        raise Exception("Unknown output extension: {}".format(ext))
    archive_path = os.path.join("target", "publish", 'dose-response')
    shutil.make_archive(archive_path, archive_format, out_path)
    print(f"Build created in: '{archive_path}.{archive_extension}'")
