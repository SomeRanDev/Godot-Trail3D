call cargo build
call cargo build --release
mv target/debug/godot_trail_3d.dll addons/SomeRanDev.Trail3D/godot_trail_3d_debug.dll
mv target/release/godot_trail_3d.dll addons/SomeRanDev.Trail3D/godot_trail_3d.dll