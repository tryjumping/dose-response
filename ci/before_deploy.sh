# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    cross rustc --target $TARGET --release --no-default-features --features "opengl cli rand fullscreen" -- -C lto

    mkdir -p $stage/"Dose Response"
    cp target/$TARGET/release/dose-response $stage/"Dose Response"
    cp target/README.md $stage/"Dose Response"/README.txt
    cp target/COPYING.txt $stage/"Dose Response"/
    echo "Version: $TRAVIS_TAG" >> $stage/"Dose Response"/VERSION.txt
    echo "Full Version: $CRATE_NAME-$TRAVIS_TAG-$TARGET" >> $stage/"Dose Response"/VERSION.txt

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
