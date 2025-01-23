# Example of building android app with iced

This is a `NativeActivity` example, based on `na-mainloop` from
[android-activity](https://github.com/rust-mobile/android-activity)


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
