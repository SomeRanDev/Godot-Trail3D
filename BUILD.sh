#!/bin/sh
cargo build
cargo build --release
mv target/debug/godot_trail_3d.so addons/SomeRanDev_Trail3D/godot_trail_3d_debug.so
mv target/release/godot_trail_3d.so addons/SomeRanDev_Trail3D/godot_trail_3d.so