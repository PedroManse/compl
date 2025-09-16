# Completion graph helper

Instead of making complex and hard to read bash scripts for my completions, I
made and use this program. It takes a list of `rules` defined as `{input} ->
{output command}[ {output arguments} ]` and inlined shell scripts (for ease of
use).

The `input` syntax wors as follows:

| text |        input       |
|------|--------------------|
| .    | Any text           |
| ?    | Maybe some text    |
| *    | Any amount of text |
|      | The text as is     |

The `output` syntax works as follows:
| text | output                            |
|------|-----------------------------------|
| word | Use the list of words             |
| sh   | Use the stdout of the script[^1]  |
| glob | Use the matched glob pattern[^1]  |
| exe  | Use the stdout of the program[^1] |
| end  | Don't respond                     |

You can inline a script with the directive `# sh {script name}` and end it with `# end`[^2]

# Example for program [nix-edit](https://github.com/PedroManse/dots/blob/fea19dca79d76b102a3102ccfc7625c93829bb11/bash/nix.bash#L8-L31):

```
[ progs ? ] -> sh[ list_nix_files ]
[ home ] -> end
[ sys ] -> end
[ ? ] -> word[ sys home progs ]

# sh list_nix_files
for path in "$DOTS"/nix/programs/*.nix ; do
        basename "${path%.nix}"
done
# end
```

Notice the order of the rules matter. In this example the 4° rule must be
defined before the third, since `[ ? ]` matches `[ sys ]`

[^1]: The words are split based on whitespace
[^2]: Semantic whitespace for script directives
