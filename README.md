# RPGMaker2kPatcher

## Usage

- `rpgmaker2kpatcher`: Shows the help menu
- `rpgmaker2kpatcher decompileMaps`: Generates TOML representations for `LcfMapUnit`s
    - Uses `PATH_TO_ORIGINAL` and `PATH_TO_REFERENCE`
- `rpgmaker2kpatcher generatePatches`: Generates TOML patches to apply to `LcfMapUnit`s
    - Uses `PATH_TO_ORIGINAL` and `PATH_TO_WORKSPACE`
- `rpgmaker2kpatcher applyPatches`: Applies TOML patches to `LcfMapUnit`s and generates patched files in another directory
    - Uses `PATH_TO_ORIGINAL` and `PATH_TO_WORKSPACE` and `PATH_TO_PATCHED`
    - Also warns of any potential issues with the patch files, such as more than 4 lines of dialogue and going over character limit.
- `rpgmaker2kpatcher convertLegacyPatches`: Convert old JSON patches to the new TOML patches

`.env` Variables
- `PATH_TO_ORIGINAL`: Root folder of the original RPGMaker2000 game.
- `PATH_TO_WORKSPACE`: Location of TOML patches. Commit this section to version control.
- `PATH_TO_REFERENCE`: Location of TOML maps. Do not commit this to version control.
- `PATH_TO_PATCHED`: Root folder of the patched RPGMaker2000 game.

## Organization

- `structs/`: The main folder to look at to understand each decoded structure
- `types/`: Assistant binrw types for `structs/`, notably 1-5 byte dynamic integer
- `wrappers/`: Assistant structures for `structs/` for complex operations (such as a u8 preceded by a 1-5 byte dynamic integer)

## Clipboard / Current Status

**Right now:**
- Use `bindiff` to verify that your new patch format can take in your old data and successfully patch it identically
- Rename `[[replace]]` to `[[other]]`, as `[[other]]` is specifically for replacing text, no flexibility for anything else.

Your next goals after that are:
- Database parsing & TOML

The next release will only have the pre-patched release, no dev stuff or separate patch generated.
- Source and destination folders.
