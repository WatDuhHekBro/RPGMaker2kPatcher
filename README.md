# RPGMaker2kPatcher

## Rewrite & Goals

If you don't want this to be stuck in development hell yet again, **SCALE DOWN YOUR PROJECT**.

- [ ] LMU to JSON (readable intermediary, should not be used in patching) - bindiff bidirectional to test
- [ ] LMU to TOML patch
    - Don't even *think* about automatic line wrapping right now if it means you get stuck in development hell!
- [ ] LMU + TOML patch = Patched LMU
- [ ] `rpgmaker2kpatcher`: Generates a `config.toml` file for setting up file paths before proceeding. Running this command with all file paths filled automatically generates a new patched version in the file path you specify. e.g. `BaseGame` fetches `../Workspace` to generate `PatchedGame`, where `../Workspace/*.lmu` and `../Workspace/Picture/*.png` exist.
    - It'll also warn you of any lines over the character limit instead of trying to come up with a fancy TUI that I never got done originally.

Would the image processing and live TUI that you never got done be nice? Absolutely. Will it grind this project to a halt? It already did. Do NOT let scope creep kill this round again.

### Current Status

For easier indexing, probably better to convert all those Vec's to HashMaps. Man... But on second thought, the array guarantees the same structure, so it remains as close to the original binary as possible, so just use less efficient helper functions to loop through the arrays.

-----

**Right now:** You just finished the TOML patch gen, now it's time to actually test it out and build the patching functionality.

Your next goals after that are:
- Database parsing & TOML
- Setup bulk functions (e.g. `util::generate_patches("/path/to/folder")`)
- Use `bindiff` to verify that your new patch format can take in your old data and successfully patch it identically

Dump
- `rpgmaker2kpatcher generatePatches`
- `.env` file for easy runtime config

## Organization

- `structs/`: The main folder to look at to understand each decoded structure
- `types/`: Assistant binrw types for `structs/`, notably 1-5 byte dynamic integer
- `wrappers/`: Assistant structures for `structs/` for complex operations (such as a u8 preceded by a 1-5 byte dynamic integer)
