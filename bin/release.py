#!/usr/bin/env python3

import boto3
import datetime
from glob import glob
import os
from pathlib import Path
import platform
import shutil
import sys


def mkdir_p(directory):
    try:
        os.makedirs(directory)
    except OSError:
        pass
    return directory


def die(message):
    print(message, file=sys.stderr)
    sys.exit(1)


def print_env(name):
    print(f"{name}: {os.environ.get(name)}")


if __name__ == '__main__':
    if len(sys.argv) < 2:
        die("You must pass the AWS S3 bucket name as the first argument.")
    bucket_name = sys.argv[1]

    env = os.environ

    target_triple = env['TARGET_TRIPLE']
    commit_hash = env['GITHUB_SHA']
    run_id = env['GITHUB_RUN_ID']
    run_number = env['GITHUB_RUN_NUMBER']
    system = platform.system()
    print_env('GITHUB_REF')
    ref_name = env.get('GITHUB_REF', '').split('/')[-1]
    archive_extension = env.get('ARCHIVE_EXT', 'tar.gz')

    # Ref formats:
    # Pull request: refs/pull/13/merge
    # Push to master: refs/heads/master
    # Tag/release: refs/tags/v2.0.0-ci-16
    if ref_name in ('master', 'main'):
        print("This is a nightly")
        nightly = True
    elif ref_name.startswith('v'):
        print(f"This is a release tag: {ref_name}")
        nightly = False
    else:
        print(f"The ref is neither a tag nor push to the main branch: '{ref_name}'")
        nightly = True
        # TODO: comment this out if you want to test release payload uploads to S3
        # TODO: remove the exit here and just skip the upload at the end
        #sys.exit(0)

    if nightly:
        releases_destination = 'nightlies'
        today = datetime.datetime.utcnow().date().isoformat()
        release_version = f"{today}-{run_number}"
    else:
        releases_destination = 'releases'
        release_version = ref_name

    exe_name = 'dose-response'
    if system == 'Windows':
        full_exe_name = f'{exe_name}.exe'
        debug_script = 'Debug.bat'
    else:
        full_exe_name = exe_name
        debug_script = 'debug.sh'

    if archive_extension == 'zip':
        archive_format = 'zip'
    elif archive_extension == 'tar.gz':
        archive_format = 'gztar'
    else:
        raise Exception("Unknown output extension: {}".format(ext))

    full_version = f'{exe_name}-{release_version}-{target_triple}'
    # Example: dose-response-v2.1.3-rc2-x86_64-pc-windows-msvc
    # NOTE: keeping this in a separate variable so if we change the archive
    # format, we don't mess up the version numbering scheme.
    archive_name = full_version
    print(f"Archive name: {archive_name}")

    target_dir = Path('target')
    release_dir = target_dir / 'release'
    publish_dir = target_dir / 'publish'
    out_dir = f'Dose Response {release_version}'
    out_path =  publish_dir / out_dir

    # Nightly URL format:
    # s3://<bucket>/nightlies/2021-05-17-232/dose-response-2021-05-17-232-x86_64-pc-windows-msvc.zip
    # Release URL format:
    # s3://<bucket>/releases/v2.1.3-rc2/dose-response-v2.1.3-rc2-x86_64-pc-windows-msvc.zip
    s3_destination_path = f'{releases_destination}/{release_version}/{archive_name}.{archive_extension}'
    print(f"S3 Destination path: {s3_destination_path}")

    mkdir_p(out_path)
    shutil.copy(release_dir / full_exe_name, out_path)

    print(f"Locale preferred encoding: {locale.getpreferredencoding()}")


    # NOTE: this converts the line endings into the current platform's expected format:
    with open("README.md", 'r', encoding='utf-8') as source:
        with open(out_path / 'README.txt', 'w') as destination:
            destination.writelines(source.readlines())
    with open("COPYING.txt", 'r', encoding='utf-8') as source:
        with open(out_path / 'LICENSE.txt', 'w') as destination:
            destination.writelines(source.readlines())
    with open("third-party-licenses.html", 'r', encoding='utf-8') as source:
        print(f"Source encoding: {source.encoding}")
        with open(out_path / 'third-party-licenses.html', 'w') as destination:
            print(f"Destination encoding: {destination.encoding}")
            lines = source.readlines()
            destination.writelines(lines)

    shutil.copy(debug_script, out_path)

    version_contents = f"Version: {release_version}\nFull Version: {full_version}\nCommit: {commit_hash}\n"
    with open(out_path / 'VERSION.txt', 'w') as f:
        f.write(version_contents)

    print("Adding icons...")
    icons_destination_path = out_path / 'icons'
    mkdir_p(icons_destination_path)
    for filename in glob('assets/icon*'):
        shutil.copy(filename, icons_destination_path)

    # NOTE: `shutil.make_archive` will provide the archive extension, don't pass it in the filename
    archive_path = target_dir / 'publish' / archive_name
    shutil.make_archive(archive_path, archive_format, publish_dir, out_dir)
    archive_full_file_path = f'{archive_path}.{archive_extension}'
    print(f"Build created in: '{archive_full_file_path}'")

    # TODO: Commenting this out, we want to test the release script in the PR so we can iterate, but not actually upload stuff
    # s3 = boto3.resource('s3')
    # s3.Bucket(bucket_name).upload_file(archive_full_file_path, s3_destination_path)
