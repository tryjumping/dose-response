set -ex

# Run the test builds as well as any actual tests.
main() {
    local extra_features=

    case $TRAVIS_OS_NAME in
        linux)
            extra_features="linux-extra-features"
            ;;
        osx)
            extra_features="macos-extra-features"
            ;;
    esac

    echo "Test environment:"
    env

    # NOTE: the `sdl-static-link` feature breaks the build on macOS so we've disabled it for now.
    # TODO: investigate how to fix it.
    cross build --features "test" --target $TARGET
    cross build --features "test" --target $TARGET --release
}

if [ -z $TRAVIS_TAG ]; then
    echo "This is not a release tag. Running test builds."
    main
else
    echo "This is a release tag: $TRAVIS_TAG. Skipping test builds."
fi
