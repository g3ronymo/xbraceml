# xbraceml
Makes writing xml less redundant. 
Convert a simple markup into xml. 
Extensible via easy to write and language independent plugins.

Convert `\hello{world}` to `<hello>world</hello>`. 

## Syntax

Element: `\name attributes{content}`
    Name is required. Attributes are optional if present they are separated 
    from the name by on whitespace.
    { and } are required but content can be empty.

Special Sequence:

- \% everything in between two \% is left unchanged.

Special Elements:

- `\$o{} produces a {`
- `\$c{} produces a }`
- `\$s{} produces a \\`
- `\${} is simply removed, can be used for comments`

Special Elements can be disabled with the -d option.

## Plugins

Plugins can be added with the `-p PATH` option. PATH can be a directory or
executable file. If path is a directory all executable files in that directory
are treated as plugins. 

If a plugin is executed with `elements` as argument, it should print the 
name of all elements it can handle to stdout, seperated by a space.

If xbraceml encounters a element that is supported by a plugin
it executes the plugin and writes to its stdin: 

1. name
2. `\r\n\r\n`
3. attributes 
4. `\r\n\r\n`
5. content

The entire element is then replaced with wathever the plugin writes to stdout.



