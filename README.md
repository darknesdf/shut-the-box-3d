# Shut The Box 3D

A semi-3D "Shut the Box" game made with **Godot 4.3** (GDScript, GL Compatibility renderer).

- 3D dice with procedurally drawn pips, thrown in a smooth arc with damped bounces and a spin that winds down onto the rolled value
- Wooden 3D tray, numbered cards 1-9 that pop out when selected and fold down with an ease-in drop and soft rebounds
- **Lost screen**: when no combination fits, the remaining tiles turn red and slump, the camera shakes, and a "YOU LOST" panel shows your score
- **Win screen** with confetti when you shut the box
- **Clear** button to un-pick tiles, stats for best score and wins / games played (saved locally)
- Haptic feedback on Android, gentle camera sway, shadows
- Touch / mouse controls, landscape layout

## How to play

1. Tap **ROLL**. (When tiles 7, 8 and 9 are all shut, only one die is rolled.)
2. Tap tiles whose numbers add up to the roll. When the total matches, they fold down automatically.
3. Shut all 9 tiles to win. If no combination is possible, the game ends and your score is the sum of the open tiles (lower is better).

## Build on GitHub

1. Create a new repository and push this folder to it.
2. Open the **Actions** tab - the **Build Android APK** workflow runs on every push.
3. Download the APK from the run's **Artifacts** (`ShutTheBox3D-apk`).
4. Tag a commit `v1.0` to also attach the APK to a GitHub Release.

The workflow installs Godot 4.3 + export templates, the Android SDK, creates a debug keystore and exports a debug-signed APK (`com.example.shutthebox3d`). Change the package name in `export_presets.cfg`.

## Run locally

Open the folder in Godot 4.3+ and press F5.

## Layout

- `scripts/main.gd` - game rules, world, UI
- `scripts/die.gd` - 3D die + roll animation
- `scripts/tile.gd` - numbered tile + select / flip animations
