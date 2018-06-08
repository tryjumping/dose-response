#!/usr/bin/env python

from ConfigParser import SafeConfigParser
import os
import shutil
from sys import argv
import tarfile
import tempfile
import urllib


def noop(*args):
    pass


def add_lib(target, version, sources):
    shutil.copytree(os.path.join('lib', target['triple']),
                        os.path.join(sources, 'Dose Response', 'lib'))


TARGETS = [
    {
        'triple': 'x86_64-unknown-linux-gnu',
        'extension': 'tar.gz',
        'process': add_lib,
        'platform-name': 'linux64',
    },
    {
        'triple': 'x86_64-pc-windows-msvc',
        'extension': 'zip',
        'process': noop,
        'platform-name': 'win64',
    },
    {
        'triple': 'x86_64-apple-darwin',
        'extension': 'tar.gz',
        'process': noop,
        'platform-name': 'osx64',
    },
]


def input_filename(target, version):
    return "dose-response-{version}-{triple}.tar.gz".format(
        version=version,
        triple=target['triple'])


def remote_url(target, version):
    return ("https://github.com/tomassedovic/dose-response/releases/download/"
           "{version}/dose-response-{version}-{triple}.tar.gz").format(
        version=version,
        triple=target['triple'],
        extension=target['extension']
        )


def download_file(source_url):
    temp = tempfile.mkstemp()
    path = temp[1]
    downloaded_path, result = urllib.FancyURLopener().retrieve(source_url, path)
    assert downloaded_path, path
    # NOTE: it seems that when the download succeeds, the `status` can be `None`
    status = result.get('status')
    downloaded_ok = status is None or status.startswith('200')
    if downloaded_ok:
        return path
    else:
        print "Error retrieving URL: {}".format(result.get('status'))
        return None


def extract_file(source_path):
    destination_dir = tempfile.mkdtemp("dose-response");
    tar = tarfile.open(source_path, "r:gz")
    tar.extractall(path=destination_dir)
    return destination_dir


def output_filename(target, version):
    return "dose-response-{version}-{platform}".format(
        version=version,
        platform=target['platform-name'])



def package_sources(target, version, sources):
    out_path = os.path.join("target", "publish", version,
                                output_filename(target, version))
    mkdir_p(out_path)
    ext = target['extension']
    if ext == 'zip':
        archive_format = 'zip'
    elif ext == 'tar.gz':
        archive_format = 'gztar'
    else:
        raise Exception("Unknown output extension: {}".format(ext))
    shutil.make_archive(out_path, archive_format, sources)
    return '{}.{}'.format(out_path, ext)


def mkdir_p(directory):
    try:
        os.makedirs(directory)
    except OSError:
        pass
    return directory


if __name__ == '__main__':
    if len(argv) > 1:
        version = argv[1]
    else:
        cargo_toml = SafeConfigParser()
        cargo_toml.readfp(open('Cargo.toml'))

        # NOTE: the string comming out of the file is quoted, so we drop the first
        # and last characters:
        version = cargo_toml.get('package', 'version')[1:-1]
        # NOTE: the versions on github have the `v` prefix:
        version = "v" + version

    for target in TARGETS:
        src_url = remote_url(target, version)
        src_filename = input_filename(target, version)
        #input_directory = os.path.join("target", "publish", version, 'in')
        #mkdir_p(input_directory)
        #input_destination = os.path.join(input_directory, src_filename)
        print "Downloading file:", src_url
        #print "to:", input_destination
        downloaded_path = download_file(src_url)
        if not downloaded_path:
            continue
        print "Downloaded to:", downloaded_path
        sources = extract_file(downloaded_path)
        print "Extracted to: '{}'".format(sources)
        target['process'](target, version, sources)
        artifact_path = package_sources(target, version, sources)
        print "Build created in: '{}'".format(artifact_path)
        shutil.rmtree(sources)
        os.remove(downloaded_path)
        print '---'
