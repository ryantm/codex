# Headers, primary, and secondary text

- **Headers:** Use `bold`. For markdown with various header levels, leave in the `#` signs.
- **Primary text:** Default.
- **Secondary text:** Use `style::secondary_text_style()` for shared footer descriptions and
  status lines. It resolves a readable muted foreground when the terminal palette is known
  and clears inherited dim/bold modifiers. Other secondary surfaces may still use `dim`.

- **Keyboard hints:** Use `key_hint` span helpers for compact, uniformly bold shortcut labels
  and readable secondary descriptions. Include `+`, `/`, and chord separators in the label's
  emphasis. The `?` shortcut reference uses readable accent-colored keys
  without bold. Separate composer footer actions with `·`.

# Foreground colors

- **Default:** Most of the time, just use the default foreground color. `reset` can help get it back.
- **Active controls and input emphasis:** Use `style::accent_style()` or
  `style::accent_color_on(background)` for text on a painted surface.
- **Semantic statuses:** Use `style::status_style(StatusTone)`; terminal-owned ANSI
  colors remain the fallback for limited palettes.
- **Success and additions:** Use ANSI `green`.
- **Errors, failures and deletions:** Use ANSI `red`.
- **Codex:** Use ANSI `magenta`.

# Avoid

- Avoid unchecked custom foreground colors. Use `style::readable_color_on(color, background)`
  for theme-derived text, passing the background actually painted behind it. `None` means
  the terminal background; color reduction is included in the contrast check. When the
  background is unknown, supported theme colors are preserved rather than guessed.
  Generic code renderers preserve configured syntax foregrounds; diff rendering resolves
  them against its add/delete fills.
- Avoid ANSI `black` & `white` as foreground colors because the default terminal theme color will do a better job. (Use `reset` if you need to in order to get those.) The exception is if you need contrast rendering over a manually colored background.
- Avoid ANSI `blue` and `yellow` because for now the style guide doesn't use them. Prefer a foreground color mentioned above.

(There are some rules to try to catch this in `clippy.toml`.)

# Menu descriptions

- Prefer concise built-in option descriptions and disabled-option explanations without trailing
  periods.
- When a description needs multiple sentences, keep normal punctuation, including the final period.
  Do not combine sentences at the expense of clarity.
- Preserve externally supplied descriptions as written.

# Terminal palette changes

On Unix, terminals that answer the startup OSC 10/11 probe are sampled every two
seconds while Codex owns the terminal. Changes to the default foreground or
background update the shared palette and request a redraw, so palette-derived
styles such as the composer background follow live terminal theme changes,
including through SSH and multiplexers that forward color queries.

Runtime queries use crossterm's shared parser to retain interleaved keys and
bracketed paste. Queries pause during editor and suspend handoffs, and terminal
restoration waits for an in-flight query. Rendering continues while querying;
input can be delayed by the query round trips. A failed query retains the last
complete palette and stops further sampling for that TUI (crossterm's timeout is
two seconds per color). Terminals without a startup palette keep their existing
fallback behavior. The separately selected `/theme` syntax theme is unchanged.
