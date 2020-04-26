# bubbleprompt

Generator for "bubbly" shell prompts:

<img width="235" height="74" src="example.png">

Works with Bash and Zsh. Requires PragmataPro font.

## Usage

Build and install binary with Cargo:

```
$ cargo install --path .
```

Call the binary from your profile:

```shell
# For Zsh
PROMPT=$(bubbleprompt --shell zsh '{0,6:ZSH {0,15:%~}} ')

# For Bash
PS1=$(bubbleprompt --shell bash '{0,3:BASH {0,15:\w}} ')
```

### Template string

The template string can contain any text. Colored sections are specified with this syntax:

```
{fg,bg:text}
```

Sections can be nested. Colors should be in `0..255` range.
