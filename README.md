pacfiles
====

pacfiles is a `pacman -F` alternative that runs blazingly fast. It archieves this by using [plocate](https://plocate.sesse.net/) databases.

Installation
----

It depends on `libarchive`, `pacman` and `plocate`. You can install this with:

```
cargo install pacfiles
```

Usage
----

It has a subset of options from `pacman -F`:

```
A pacman -F alternative that runs blazingly fast

Usage: pacfiles [OPTIONS] [QUERY]...

Arguments:
  [QUERY]...  The query; unlike pacman, globs (*?[]) are supported in non-regex mode

Options:
  -F, --files       ignored
  -l, --list        List the files owned by the queried package
  -x, --regex       Interpret each query as a POSIX extended regular expression
  -q, --quiet       Do not output colors and file paths
  -y, --refresh...  Refresh & rebuild databases; give twice to force
  -h, --help        Print help
  -V, --version     Print version
```

It tries to output in the same format as pacman.

Performance
----

Search for a file:

```
>>> time pacman -F vim
extra/gvim 9.1.1165-1
    usr/bin/vim
extra/radare2 5.9.8-1
    usr/share/doc/radare2/vim
    usr/share/radare2/5.9.8/magic/vim
extra/rizin 0.7.4-1
    usr/share/rizin/magic/vim
extra/vim 9.1.1165-1
    usr/bin/vim
archlinuxcn/gvim-lily 9.1.1163-1 [已安装: 9.1.1144-1]
    usr/bin/vim
archlinuxcn/vim-lily 9.1.1163-1
    usr/bin/vim
chaotic-aur/cheat 4.4.2-3
    usr/share/cheat/cheatsheets/community/vim
chaotic-aur/neovim-drop-in 1-1.1
    usr/bin/vim
chaotic-aur/neovim-symlinks 5-1
    usr/bin/vim
chaotic-aur/radare2-git 5.9.8.r455.ge75c95a-1
    usr/share/doc/radare2/vim
    usr/share/radare2/5.9.9/magic/vim
== TIME REPORT FOR pacman -F vim ==
   User: 3.24s  System: 1.17s  Total: 4.423s
   CPU:  99%    Mem:    2935 MiB

>>> time pacfiles -F vim
extra/gvim 9.1.1165-1
    usr/bin/vim
extra/radare2 5.9.8-1
    usr/share/doc/radare2/vim
    usr/share/radare2/5.9.8/magic/vim
extra/rizin 0.7.4-1
    usr/share/rizin/magic/vim
extra/vim 9.1.1165-1
    usr/bin/vim
archlinuxcn/gvim-lily 9.1.1163-1 [installed: 9.1.1144-1]
    usr/bin/vim
archlinuxcn/vim-lily 9.1.1163-1
    usr/bin/vim
chaotic-aur/cheat 4.4.2-3
    usr/share/cheat/cheatsheets/community/vim
chaotic-aur/neovim-drop-in 1-1.1
    usr/bin/vim
chaotic-aur/neovim-symlinks 5-1
    usr/bin/vim
chaotic-aur/radare2-git 5.9.8.r455.ge75c95a-1
    usr/share/doc/radare2/vim
    usr/share/radare2/5.9.9/magic/vim
== TIME REPORT FOR pacfiles -F vim ==
   User: 5.09s  System: 0.13s  Total: 0.378s
   CPU:  1383%  Mem:    14 MiB
```

List package contents:

```
>>> time pacman -Fl vim >/dev/null
== TIME REPORT FOR pacman -Fl vim > /dev/null ==
   User: 0.94s  System: 0.31s  Total: 1.260s
   CPU:  99%    Mem:    821 MiB

>>> time pacfiles -Fl vim >/dev/null
== TIME REPORT FOR pacfiles -Fl vim > /dev/null ==
   User: 0.02s  System: 0.02s  Total: 0.039s
   CPU:  99%    Mem:    14 MiB
```
