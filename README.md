# Godot Trail 3D (in Rust)

Adds a Trail3D node to Godot.
It is compatible with GDScript, C#, and Rust!

This is ported from @axilirate's gist.
This version is rewritten in Rust and optimized to remove the expensive list manipulations.
https://gist.github.com/axilirate/96a3e77d597c2527582dbc79aecbab70

@axilirate's version is a Godot 4 port of a Node from Oussama BOUKHELF's Godot 3 Trail System:
https://github.com/OBKF/Godot-Trail-System
https://github.com/OBKF/Godot-Trail-System/blob/master/LICENSE

I also added support for a color gradient inspired by [KindoSaur's YouTube video](https://www.youtube.com/watch?v=vKrrxKS-lcA).

# Installation

1) Install the [Rust Programming Language](https://www.rust-lang.org/) if you don't have it (you can check if you have it by running `cargo` on the command prompt).

2) Download this repo.

3) Run `BUILD.bat` (on Windows) or `BUILD.sh` (on Linux).

5) Finally, copy the `addons/SomeRanDev_Trail3D/` folder into your project's `addons/` folder.
