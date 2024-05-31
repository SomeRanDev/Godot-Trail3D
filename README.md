# Godot Trail 3D

Adds a Trail3D node to Godot.

It is compatible with GDScript, C#, and Rust!

<img src="https://github.com/SomeRanDev/Godot-Trail3D/blob/main/demo.gif"/>

## Credits

This is a port of [@axilirate](https://github.com/axilirate)'s [gist](https://gist.github.com/axilirate/96a3e77d597c2527582dbc79aecbab70).<br>
This version is rewritten from GDScript to Rust, and it's optimized to remove the expensive list manipulations.

[@axilirate](https://github.com/axilirate)'s version is a Godot 4.0 port from [Oussama BOUKHELF](https://github.com/OBKF)'s [Godot 3 Trail System](https://github.com/OBKF/Godot-Trail-System)<br>
You can view and comply with their license [here](https://github.com/OBKF/Godot-Trail-System/blob/master/LICENSE).

I also added support for a color gradient inspired by [KindoSaur's YouTube video](https://www.youtube.com/watch?v=vKrrxKS-lcA).

## Installation

1) Install the [Rust Programming Language](https://www.rust-lang.org/) if you don't have it (you can check if you have it by running `cargo` on the command prompt).

2) Download this repo.

3) Run `BUILD.bat` (on Windows) or `BUILD.sh` (on Linux).

5) Finally, copy the `addons/SomeRanDev_Trail3D/` folder into your project's `addons/` folder.

## How to Use

The `Trail3D` node should not be moved directly; instead, add it as a child to a mobile node.

To make the colors work, add a `Material Override` and enable `Vertex Color -> Use as Albedo` (or read from `COLOR` in a shader material).

If you want transparency, you also need to enable it in the material.
