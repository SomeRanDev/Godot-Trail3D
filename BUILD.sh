#!/bin/sh
cargo build
cargo build --release
mv target/debug/libgodot_trail_3d.so addons/SomeRanDev_Trail3D/libgodot_trail_3d_debug.so
mv target/release/libgodot_trail_3d.so addons/SomeRanDev_Trail3D/libgodot_trail_3d.so

