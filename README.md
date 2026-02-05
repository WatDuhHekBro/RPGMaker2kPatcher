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

## Clipboard / Current Status

**Right now:** In the middle of modifying `file_operations` to include database patch, then I need to test out the database patch for myself.
- Don't do any ext = ldb, just hardcode it. Assume that there's only one database with the same name each time.
- TODO: importLegacyPatches
- TODO: applyPatches

Database
- TOML patch
    - Create a `[[database-patch]]` field if necessary, I really don't want to have a separate patch format.
    - Then again, it can't be that hard to create two Patch formats. Just not semantic with `.patch.toml`.
    - `header = 21`?
    - Current Idea: `page` becomes optional, though expected for maps. Database assumes header 25 header 21 for `[[dialogue]]` and `[[text]]`. Anything outside of that you need to use a different method, the arbitrary data editing method.
    - `[[arbitrary]]` -> `path = [21, 114]` (still takes offsets into accounts, especially to replace events)

What exactly needs to get patched in the database?
- 21 (Vocabulary, only other header listed in original patch)
- 25 (EventCommands)
- Basically everything outside of 25 is on a case-by-case basis, as you need it.

Your next goals after that are:
- Port over manual patches functionality
    - Maybe the patch will have an arbitrary path to follow for really jank patches on both maps and databases, be as flexible as possible
    - `arbitrary_path = [23, 1]`
- Improved patch format (See EasyRPG Editor to help)
    - See if you can remove some `indent` fields by inferring from branching/logic commands like `12010` (branch if)
    - See if you can infer `is_portrait` that applies to Aedemphia as well
        - If you can, then you can add an automatic line wrap option, basically meaning the position isn't important for this dialogue box

-----

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

```
Bulk converting legacy patches...
WARNING: [map.Map0074.event.7.page.1.command.94] found no equivalent dialogue in its legacy patch!
WARNING: [map.Map0074.event.7.page.1.command.144] found no equivalent dialogue in its legacy patch!
WARNING: [map.Map0084.event.12.page.1.command.24] found no equivalent dialogue in its legacy patch!
WARNING: [map.Map0084.event.15.page.1.command.9] found no equivalent dialogue in its legacy patch!
WARNING: [map.Map0084.event.15.page.1.command.54] found no equivalent dialogue in its legacy patch!
WARNING: Some legacy entries weren't used in the conversion process for Map0179!
{(25, 2, 157): "Bedrohung_von_oben"}
WARNING: Some legacy entries weren't used in the conversion process for Map0208!
{(34, 10, 187): "2003MaximumBattle"}
ERROR: Error on reading TOML patch file for Map0250!
ERROR: Error on reading TOML patch file for database!
```
