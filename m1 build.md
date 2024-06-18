1. README.md in main catalog, look at `Compiling Programs` section
2. I had to:
   1. `cargo update` (I didn't update earlier)
   2. `rustup default stable-x86_64-apple-darwin`
3. Then I had a problem with blake3 building script, so I followed instructions in https://solana.stackexchange.com/questions/6987/mac-platform-tools-1-16-cargo-build-sbf-error-failed-to-run-custom-build :
   1. `xcode-select --install`
   2. `brew install gcc@12`
   3. `export CPATH="/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include"`
4. Worked
