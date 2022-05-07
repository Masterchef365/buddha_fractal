channel() {
    cargo run --release -- --disc 3.0 -i 400000000 -w 1920 -h 1080 --div 4 $@
}

channel -s 2500 blue.png
channel -s 1000 green.png
channel -s 500 red.png
