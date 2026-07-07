# Send ^C to dbobs screen to stop current dbobs session, if any
screen -S dbobs -X stuff $'\003'

# Build dbobs
cargo build --release

# Run dbobs in screen
/usr/bin/screen -S dbobs -d -m bash -c "cargo run --release" ; exit
