# Aella

Your friendly neighborhood grammar checker, built for speed and simplicity.

## What's This?

Aella is a blazing fast grammar checking app powered by `harper-core`. Whether you prefer a sleek desktop experience or the command line, we've got you covered.

### Backstory

I work 5 to 6 days at week as a developer, constantly changing code, adding features, well, the usual.
I'm always opening PRs, and as a good professional that I'm trying to be; I try to sound good in description, try to be concise, and so on.
From 0 to 50 times a day I go to ChatGPT and say something like `fix grammar of this: <Enter PR description` I usually don't commit many mistakes while typing but I type fast and I can easily eat an apostrophe, a comma, something. So using ChatGPT for this all the time is inefficient, it might not be that slow, but it's slower compared to this.
So on my free time I created Aella (A random Greek name I liked), a simple app that checks grammar offline (Only English at the moment), fast, and you just keep your tone. For me it lets me keep my ideas over a paragraph.
For instance, I type my PR description, copy it to clipboard then in a terminal `echo "$pbpaste" | aella check | pbcopy`, done. Or I can use the Awesome Icy Desktop App.

## Features

- **Real-time Grammar Checks**: Catch mistakes as you type.
- **Markdown Support**: Write in your favorite format.
- **Auto-Save History**: Never lose a thought (backed by Turso).
- **Search**: Find old conversations instantly.

## Look and Feel

![Main View](./.github/assets/main.avif)

![Shortcuts Modal](./.github/assets/shortcuts.avif)

![Trashed View](./.github/assets/trashView.avif)

## The Goods

- **`aella-desktop`**: The shiny GUI (built with `iced`).
- **`aella-cli`**: For the terminal lovers. Fast, powerful, scriptable.
- **`aella-core`**: The brains behind both, shared grammar engine and database.

## Getting Started

### Desktop App

Make sure you have Rust installed, then:

```bash
cargo run -p aella-desktop
```

### CLI Usage

The CLI is your quick grammar buddy for the command line. Here's everything it can do:

#### Fix Text on the Fly

```bash
# Pipe in your text
echo "I has a pen. She dont like it." | aella check
# Output: I have a pen. She doesn't like it.

# Check a file
aella check essay.txt

# Save the corrected version
aella check draft.txt -o polished.txt
```

#### See What Changed

```bash
# Show a colorful diff
aella check messy.txt --diff
```

Red lines show what was wrong, green lines show the fix. Simple.

#### Get Machine-Readable Output

```bash
# JSON output (great for scripting)
aella check document.txt --json
```

You get the original, corrected text, number of fixes, and how long it took.

#### Process Multiple Files at Once

```bash
# Fix all markdown files
aella check "*.md" --batch

# Save them to a new folder
aella check "*.txt" --batch -o corrected/
```

It'll cruise through each file and show you what it's doing.

#### Save Your Work to History

```bash
# Store the full text in the database
aella check important.txt --save-content
```

By default, Aella just tracks metadata (how many fixes, when you ran it). Add `--save-content` to keep the full before/after text.

#### Check Your History

```bash
# See your last 20 runs
aella history

# Or show more
aella history --limit 50
```

Every check gets logged, see what you've been fixing and when.

#### Quick Reference

```bash
aella check [FILE]              # Check file or stdin
  -o, --output FILE             # Save to file
  --diff                        # Show changes as diff
  --json                        # Output as JSON
  --save-content                # Store full text in DB
  --batch                       # Process multiple files

aella history                   # Show recent runs
  --limit N                     # Number of runs to show
```

### MacOS

You can use cargo-bundle:

```bash
cargo bundle -p aella-desktop --release
```

That's it. Happy writing! 📝
