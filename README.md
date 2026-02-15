# RPGMaker2kPatcher

## How To Use

***🚧 TODO: Under Construction 🚧***

Dump
```
How much documentation should be in rpgmaker2kpatcher vs Velsarbor? Specific to project or general?
- Rpgmk readme: What each option does specifically and why you'd want to do it.
- Rpgmk general workflow section (not just commands list)
- Rpgmk: Short description for GitHub and top of readme
- Rpgmk: Move command reference down a section
- Spacing in sections
- [ ] Documentation is just add what this is all about to an outside observer
- Patch fields, what can be added, what the purpose is of each one (also stuff like has_portrait being optional and it doesn't affect how the text renders by default)
```

## CLI Usage

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
    - Optionally uses `PATH_TO_EXTRACTED_TEXT`
- `rpgmaker2kpatcher importLegacyPatches`: Convert old JSON patches to the new TOML patches
    - Uses `PATH_TO_WORKSPACE` and `PATH_TO_WORKSPACE_LEGACY`

`.env` Variables
- `PATH_TO_ORIGINAL`: Root folder of the original RPGMaker2000 game.
- `PATH_TO_REFERENCE`: Location of TOML maps. Do not commit this to version control.
- `PATH_TO_WORKSPACE`: Location of TOML patches. Commit this section to version control.
- `PATH_TO_PATCHED`: Root folder of the patched RPGMaker2000 game.
- `PATH_TO_WORKSPACE_LEGACY`: Root folder of the patched RPGMaker2000 game.
- `PATH_TO_EXTRACTED_TEXT`: Optionally redirect the location of extracted text (instead of the `PATH_TO_WORKSPACE`).

## Code Organization

- `structs/`: The main folder to look at to understand each decoded structure
- `types/`: Assistant binrw types for `structs/`, notably 1-5 byte dynamic integer
- `wrappers/`: Assistant structures for `structs/` for complex operations (such as a u8 preceded by a 1-5 byte dynamic integer)



# Clipboard / Current Status / Goals

- Clean up documentation and usage
- If you can, then you can add an automatic line wrap option, basically meaning the position isn't important for this dialogue box
    - This option only applies if the patched line is all on one line. If it's multiline, assume manual newlines, then do error checking on `applyPatches`
    - Separate command `checkDialogue`
- Generate styled HTML file for dialogue previews, no need to mess with JS (see the `dev` branch). Much easier to look at than a TUI. Also don't need `chars.json`. `rpgmaker2kpatcher generatePreviews` or `previewDialogue`
- Dialogue overflow warnings - Probably relegate to subcommand just in case there are manual overrides you want and don't want to see the warnings each time you patch

Dialogue / Line Wrap
- Test: Default patches should be fully identical because auto line wrap is something you need to opt into by putting it all onto one line.
- Rule for ellipses, continuous punctuation String (do not split `...`)
    - Do this by having Dialogue punctuation be counted as Normal text, but act differently via `is_punctuation_mode_active` flag. As soon as punctuation returns to something non-punctuation, then it splits the text up.
    - Also cases like "word?!" and "word?!word" 

Maybe merge `previewDialogue`, `checkDialogue`, and `extractText` into one big auxiliary operation? Same logic (that you can refine by adding a `HashMap<(event, page), ...>`), roughly the same outputs (only different in specific file types, txt, html, or console warnings).
- Pass sorted HashMap as reference to 3 functions
- `checkDialogue`? Change path to extracted text to be like `PATH_TO_DIALOGUE_PREVIEW`, containing both extracted text and HTML previews.
    - `report.txt`? And if you're checking for line overflows with the HTML preview anyway... **TODO:** `report.html` with the title `Dialogue Overflow Report`. Also an easy GUI way to check what you're missing and what you can safely ignore.
    - Also make a field on Patch named `ignore_overflow = true` if you want to ignore something for the report and leave it as-is at the same time

`extractText` - Convert to ordered HashMap + use Dialogue parsing module

**TODO:** `cargo clippy`

## Edge Cases

Edge Case: Aedemphia Map0323 Event #12 Page #1 Command #59 has a control character 7F. When written with custom formatting and read back, the TOML parser throws an error. See how the default TOML formatter deals with this.
https://stackoverflow.com/questions/26741455/how-to-remove-control-characters-from-string `(str.replace(/[\u0000-\u001F\u007F-\u009F]/g, ""))`
- Then again, how often does it happen anyway? Just manually convert it to a double string literal so you can escape the control character.
- Also Map1426 Event #42 Page #1 Command #2 has the same 7F issue

Edge Case: Tara's Adventure Map1180 Event #16 Page #1 Command #23 - One line itself has a bunch of newlines. Then because the original length gets counted differently, the binary output is tangibly different because of splicing the wrong indexes.
- For this edge case, you could probably just add a `[[splice-commands]]` entry to deal with it manually.
