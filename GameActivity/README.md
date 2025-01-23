# Example of building android app with iced

This is a `GameActivity` example, based on `agdk-mainloop` from
[android-activity](https://github.com/rust-mobile/android-activity)

*Important:* there's an [issue](https://github.com/rust-mobile/android-activity/issues/79)
with event filters in emulator. To use touch screen in emulator,
you'll have to clone `android-activity` and change the default filter.

## Building and running

Check `android-activity` crate for detailed instructions.
During my tests I was running the following command and using android studio afterwards:

```bash
export ANDROID_NDK_HOME="path/to/ndk"
export ANDROID_HOME="path/to/sdk"

rustup target add x86_64-linux-android
cargo install cargo-ndk

cargo ndk -t x86_64 -o app/src/main/jniLibs/  build
```
