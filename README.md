# RPGMaker2kPatcher

## Usage

- `rpgmaker2kpatcher`: Shows the help menu
- `rpgmaker2kpatcher decompile`: Generates TOML representations for `LcfMapUnit`s
    - Uses `PATH_TO_ORIGINAL` and `PATH_TO_REFERENCE`
- `rpgmaker2kpatcher generatePatches`: Generates TOML patches to apply to `LcfMapUnit`s
    - Uses `PATH_TO_ORIGINAL` and `PATH_TO_WORKSPACE`
- `rpgmaker2kpatcher applyPatches`: Applies TOML patches to `LcfMapUnit`s and generates patched files in another directory
    - Uses `PATH_TO_ORIGINAL` and `PATH_TO_WORKSPACE` and `PATH_TO_PATCHED`
    - Also warns of any potential issues with the patch files, such as more than 4 lines of dialogue and going over character limit.
- `rpgmaker2kpatcher extractText`: Cleans and extracts all text to a separate text file.
    - Uses `PATH_TO_ORIGINAL` (for the database) and `PATH_TO_WORKSPACE`
- `rpgmaker2kpatcher importLegacyPatches`: Convert old JSON patches to the new TOML patches
    - Uses `PATH_TO_WORKSPACE` and `PATH_TO_WORKSPACE_LEGACY`

`.env` Variables
- `PATH_TO_ORIGINAL`: Root folder of the original RPGMaker2000 game.
- `PATH_TO_REFERENCE`: Location of TOML maps. Do not commit this to version control.
- `PATH_TO_WORKSPACE`: Location of TOML patches. Commit this section to version control.
- `PATH_TO_PATCHED`: Root folder of the patched RPGMaker2000 game.
- `PATH_TO_WORKSPACE_LEGACY`: Root folder of the patched RPGMaker2000 game.

## Organization

- `structs/`: The main folder to look at to understand each decoded structure
- `types/`: Assistant binrw types for `structs/`, notably 1-5 byte dynamic integer
- `wrappers/`: Assistant structures for `structs/` for complex operations (such as a u8 preceded by a 1-5 byte dynamic integer)



# Clipboard / Current Status

**Right now:** Test out porting legacy patches to your actual project now

Your next goals after that are:
- Types of manual patches
    - Splice events/pages?
- Review and prune old JS code
- Clean up documentation and usage

## Non-Immediate

- If you can, then you can add an automatic line wrap option, basically meaning the position isn't important for this dialogue box

## Edge Cases

Edge Case: Aedemphia Map0323 Event #12 Page #1 Command #59 has a control character 7F. When written with custom formatting and read back, the TOML parser throws an error. See how the default TOML formatter deals with this.
https://stackoverflow.com/questions/26741455/how-to-remove-control-characters-from-string `(str.replace(/[\u0000-\u001F\u007F-\u009F]/g, ""))`
- Then again, how often does it happen anyway? Just manually convert it to a double string literal so you can escape the control character.
- Also Map1426 Event #42 Page #1 Command #2 has the same 7F issue

Edge Case: Tara's Adventure Map1180 Event #16 Page #1 Command #23 - One line itself has a bunch of newlines. Then because the original length gets counted differently, the binary output is tangibly different because of splicing the wrong indexes.

## Plans n' Stuff

The next release will only have the pre-patched release, no dev stuff or separate patch generated.
- Source and destination folders.

```
Manual Patches
Map0081
Map0093
Map0179
Map0208
Map0224
Map0227
Map0240
Map0242
Map0245
Map0250 (Added)
```

```
Delete?
Map0006
Map0007
Map0011
Map0012
Map0014
Map0143
Map0144
Map0213
```
