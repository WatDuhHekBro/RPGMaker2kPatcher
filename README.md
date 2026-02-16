# RPGMaker2kPatcher

## How To Use

***🚧 TODO: Under Construction 🚧***

### Installation

...

### Project Setup

...

### Project Workflow

...

### Dump

```
How much documentation should be in rpgmaker2kpatcher vs Velsarbor? Specific to project or general?
- Rpgmk readme: What each option does specifically and why you'd want to do it.
- Rpgmk general workflow section (not just commands list)
- Rpgmk: Short description for GitHub and top of readme
- Rpgmk: Move command reference down a section
- Spacing in sections
- [ ] Documentation is just add what this is all about to an outside observer
- Patch fields, what can be added, what the purpose is of each one (also stuff like has_portrait being optional and it doesn't affect how the text renders by default)

Workflow:
- Make sure to binary identical, git repo, ignore_overflow if necessary or a [[splice-commands]] for leading newlines
- Basically, how to use (from existing, and from scratch - two sections)
```

## CLI Usage

- `rpgmaker2kpatcher`: Shows the help menu
- `rpgmaker2kpatcher decompile`: Generates TOML representations for `LcfMapUnit`s
    - Uses `PATH_TO_ORIGINAL` and `PATH_TO_REFERENCE`
- `rpgmaker2kpatcher generatePatches`: Generates TOML patches to apply to `LcfMapUnit`s
    - **Alias:** `rpgmaker2kpatcher generate`
    - Uses `PATH_TO_ORIGINAL` and `PATH_TO_WORKSPACE`
- `rpgmaker2kpatcher applyPatches`: Applies TOML patches to `LcfMapUnit`s and generates patched files in another directory
    - **Alias:** `rpgmaker2kpatcher apply`
    - Uses `PATH_TO_ORIGINAL` and `PATH_TO_WORKSPACE` and `PATH_TO_PATCHED`
    - Also warns of any potential issues with the patch files, such as more than 4 lines of dialogue and going over character limit.
- `rpgmaker2kpatcher preview`: Cleans and extracts all text to a separate text file, generates HTML previews of text boxes for easier debugging, and generates an HTML file named `report.html` for any overflowing lines.
    - Uses `PATH_TO_ORIGINAL` (for the database) and `PATH_TO_WORKSPACE`
    - Optionally uses `PATH_TO_PREVIEW`
- `rpgmaker2kpatcher importLegacyPatches`: Convert old JSON patches to the new TOML patches
    - Uses `PATH_TO_WORKSPACE` and `PATH_TO_WORKSPACE_LEGACY`

`.env` Variables
- `PATH_TO_ORIGINAL`: Root folder of the original RPGMaker2000 game.
- `PATH_TO_REFERENCE`: Location of TOML maps. Do not commit this to version control.
- `PATH_TO_WORKSPACE`: Location of TOML patches. Commit this section to version control.
- `PATH_TO_PATCHED`: Root folder of the patched RPGMaker2000 game.
- `PATH_TO_PREVIEW`: Optionally redirect the location of extracted text, HTML dialogue previews, and the HTML overflow report (instead of the `PATH_TO_WORKSPACE`).
- `PATH_TO_WORKSPACE_LEGACY`: Location of the old JSON patches.
- `DISABLE_DECOMPILE_INDEXES`: Disables printing array indexes in decompiled TOML, significantly useful for better git diffing for binary identical testing. If set to any value (such as `1`), it will enable this flag.

## Code Organization

- `structs/`: The main folder to look at to understand each decoded structure
- `types/`: Assistant binrw types for `structs/`, notably 1-5 byte dynamic integer



# Clipboard / Current Status / Goals

**Current Commit:** `a`

**What was I doing just now?**

**Coding:**
- Preserve `\s[06]`
- `\x[String]` except for `\n[123]` which uses actual (and it's optional function call anyway of `Option<i32>` which attempts to parse the number)

-----

- Clean up documentation and usage
- If you can, then you can add an automatic line wrap option, basically meaning the position isn't important for this dialogue box
    - This option only applies if the patched line is all on one line. If it's multiline, assume manual newlines, then do error checking on `applyPatches`
    - Separate command `checkDialogue`
- Generate styled HTML file for dialogue previews, no need to mess with JS (see the `dev` branch). Much easier to look at than a TUI. Also don't need `chars.json`. `rpgmaker2kpatcher generatePreviews` or `previewDialogue`
- Dialogue overflow warnings - Probably relegate to subcommand just in case there are manual overrides you want and don't want to see the warnings each time you patch

Dialogue / Line Wrap
- Test: Default patches should be fully identical because auto line wrap is something you need to opt into by putting it all onto one line.
    - I have a feeling there might be some weird edge case with a long single line. *Maybe `ignore_overflow` in `applyPatches` should also disable line splitting?* After all, if you're taking it out of the overflow report, you're basically dealing with it manually. Think of it as a manual override for single line dialogues.
- Rule for ellipses, continuous punctuation String (do not split `...`)
    - Do this by having Dialogue punctuation be counted as Normal text, but act differently via `is_punctuation_mode_active` flag. As soon as punctuation returns to something non-punctuation, then it splits the text up.
    - Also cases like "word?!" and "word?!word" 

Maybe merge `previewDialogue`, `checkDialogue`, and `extractText` into one big auxiliary operation? Same logic (that you can refine by adding a `HashMap<(event, page), ...>`), roughly the same outputs (only different in specific file types, txt, html, or console warnings).
- Pass sorted HashMap as reference to 3 functions
- `checkDialogue`? Change path to extracted text to be like `PATH_TO_DIALOGUE_PREVIEW`, containing both extracted text and HTML previews.
    - `report.txt`? And if you're checking for line overflows with the HTML preview anyway... **TODO:** `report.html` with the title `Dialogue Overflow Report`. Also an easy GUI way to check what you're missing and what you can safely ignore.
    - Also make a field on Patch named `ignore_overflow = true` if you want to ignore something for the report and leave it as-is at the same time
    - `Map0013: Event #119 Page #2`
    - **How about this?** Or just change all instances of "extracted text" to just "preview(s)". `extracted-text` = `preview(s)`. That is an accurate statement after all.

`extractText` - Convert to ordered HashMap + use Dialogue parsing module

**Main Functionality TODO:**
- Check if binary identical
    - Need to add edge case of trailing newline in actual text for consistency
- `ignore_overflow` disables line splitting?
- Dialogue punctuation

**TODO:** `cargo clippy`

More
- Dialogue only splits if it's one line, so by setting `ignore_overflow` on, you both ignore the error as well as preserve that one line property.
    - Maybe trailing newline because auto vs manual line?
    - So like `disable_auto_splitting` field calculated on Patch read

Basically:
```toml
# This is an auto-generated single line
patched = '''
Single line.
'''
# While this is an opt-in single line. Programmatic difference of trailing newline.
patched = '''Single line.'''
# Recommended to opt-in to all line wrap so you can preview it to see how it looks before you ship it.
```

## Edge Cases

Edge Case: Tara's Adventure Map1180 Event #16 Page #1 Command #23 - One line itself has a bunch of newlines. Then because the original length gets counted differently, the binary output is tangibly different because of splicing the wrong indexes.
- For this edge case, you could probably just add a `[[splice-commands]]` entry to deal with it manually.
