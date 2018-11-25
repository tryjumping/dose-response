# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage= \
          extra_features=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            extra_features=""
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            extra_features="sdl-static-link"
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    cross rustc --target $TARGET --release --no-default-features --features "glium-backend sdl-backend cli rand fullscreen $extra_features" -- -C lto

    mkdir -p $stage/"Dose Response"
    cp target/$TARGET/release/dose-response $stage/"Dose Response"
    # NOTE(shadower): we're bundling things statically now, don't upload the full build directory anymore:
    #cp -r target/$TARGET/release/build $stage/"Dose Response"
    cp README.md $stage/"Dose Response"/README.txt
    cp COPYING.txt $stage/"Dose Response"/LICENSE.txt
    echo "Version: $TRAVIS_TAG" >> $stage/"Dose Response"/VERSION.txt
    echo "Full Version: $CRATE_NAME-$TRAVIS_TAG-$TARGET" >> $stage/"Dose Response"/VERSION.txt
    echo "Commit: $TRAVIS_COMMIT" >> $stage/"Dose Response"/VERSION.txt

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
