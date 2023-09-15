set -e
RUST_ASSETS_DIR=target/thumbv6m-none-eabi/debug/examples
OUR_EXAMPLE_NAME=p7

MICROPYTHON_ASSETS_DIR=mp
MICROPYTHON_BIN=flash.bin
MICROPYTHON_LIBS="
CE-PiicoDev-Unified/min/PiicoDev_Unified.py
CE-PiicoDev-VL53L1X-MicroPython-Module/min/PiicoDev_VL53L1X.py
"
THEIR_EXAMPLE_NAME=CE-PiicoDev-VL53L1X-MicroPython-Module/main.py

pushd $RUST_ASSETS_DIR
echo Building ours
cargo build --target thumbv6m-none-eabi --example $OUR_EXAMPLE_NAME
echo Downloading ours to target
probe-rs download --protocol swd --chip RP2040 $OUR_EXAMPLE_NAME
echo Collecting our results
ours="$(probe-rs run --protocol swd --chip RP2040 \
  --disable-progressbars --no-location \
  $OUR_EXAMPLE_NAME 2>/dev/null | head | shuf)"
echo "$ours"
popd

pushd $MICROPYTHON_ASSETS_DIR
echo Downloading MicroPython to target
probe-rs download --chip-erase --protocol swd --chip RP2040 \
  --format bin --base-address 0x10000000 $MICROPYTHON_BIN
echo Resetting target
probe-rs reset --protocol swd --chip RP2040
dev=$(mpremote devs | grep MicroPython | cut -f1 -d' ')
echo Spamming intepreter connection until it is ready
until timeout --signal=TERM 1 mpremote connect $dev eval True >/dev/null 2>/dev/null
do
  :
done
echo Copying MicroPython libraries to target
mpremote connect $dev cp $MICROPYTHON_LIBS :
echo Collecting their results
theirs="$(mpremote connect $dev run \
  $THEIR_EXAMPLE_NAME 2>/dev/null | head | shuf)"
echo "$theirs"
popd
