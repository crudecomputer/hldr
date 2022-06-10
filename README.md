# Placeholder

*Simple, declarative data seeding for PostgreSQL*

**Important:** Placeholder is in quite the alpha state and still very feature-incomplete.

Placeholder strives to make generating _simple_ data much more succinct
and cleaner than using SQL, PL/pgSQL, or even other programming languages
with dedicated fixtures and factory libraries.

With some easy to read syntax and a single `hldr` command,
you can have a well populated database in no time without setting up
languages, dependencies, or verbose factory classes.

See the corresponding [VS Code extension](https://github.com/kevlarr/vscode-hldr)
(also in an alpha state) for syntax highlighting examples.

## Contents

1. [Installation](#installation)
2. [Usage](#usage)
   1. [Command-line options](#options)
   2. [The options file](#the-options-file)
3. [Features](#features)
   1. [General syntax](#general-syntax)
   2. [Literal values](#literal-values)
   3. [Comments](#comments)
   4. [Quoted identifiers](#quoted-identifiers)
   5. [Named records](#named-records)
   6. [References](#references)
   7. [Table aliases](#table-aliases)
4. [Planned features](#planned-features)

## Installation

Placeholder currently must be compiled from source but precompiled
binaries for common platforms should be [available soon](https://github.com/kevlarr/hldr/issues/16).

## Usage

Placeholder is designed to be easy to use.
Run `hldr --help` or `hldr -h` to see usage and all available options.

```
USAGE:
    hldr [OPTIONS]

OPTIONS:
    -c, --database-conn <CONN>     Database connection string, either key/value pair or URI style
        --commit                   Commit the transaction
    -f, --data-file <DATA-FILE>    Path to the .hldr data file to load [default: place.hldr if not
                                   specified in options file]
    -h, --help                     Print help information
    -o, --opts-file <OPTS-FILE>    Path to the optional .toml options file [default: hldr-opts.toml]
    -V, --version                  Print version information
```

### Options

Ultimately, there are **3 things** to care about.

#### 1. The data file to load

By default, `hldr` will look for a file called `place.hldr` to load,
but any other file can be loaded with the `--data-file <path>` or `-f <path>` option.

```bash
# Load the `place.hldr` file by default
$ hldr

# Or specify a different file
$ hldr --data-file example.hldr
$ hldr -f ../example.hldr
```

#### 2. The database connection

To specify database connection details, pass either key-value pair or
URI-style string via `--database-conn` or `-c`.
For available options, see the
[postgres driver docs](https://docs.rs/postgres/latest/postgres/config/struct.Config.html).
In general, options are similar to `libpq`.

```bash
# URI style
$ hldr --database-conn "postgresql://user:password@host:port/dbname"
$ hldr -c "postgresql://user:password@host:port/dbname"

# Key/value style - useful when including `options` eg. to set custom search path
$ hldr --database-conn "user=me password=passy options='-c search_path=schema1,schema2'"
$ hldr -c "user=me password=passy options='-c search_path=schema1,schema2'"
```

#### 3. Whether the transaction should be committed or rolled back

By default `hldr` rolls back the transaction to encourage dry-runs,
so pass the `--commit` flag to override that behavior.

```bash
$ hldr
Rolling back changes, pass `--commit` to apply

$ hldr --commit
Committing changes
```

### The options file

Specifying command-line options can be convenient (eg. when using
environment variables on CI/CD) but can be especially tedious for
local development.

To make life easier, the database connection and default file can be
specified in a `hldr-opts.toml` file.

```toml
# hldr-opts.toml
#
# None of these values are required, and if supplied they will be overridden
# by any command-line options present

data_file = "../some-custom-file.hldr"
database_conn = "user=me password=passy options='-c search_path=schema1,schema2'"
```

If for whatever reason `hldr-opts.toml` is a disagreeable name,
a custom options file can be specified.

```bash
$ hldr --opts-file ../path/to/file.toml
$ hldr -o ../path/to/file.toml
```

**Important:** As this file can be environment-dependent and contain sensitive
details, it **should not be checked into version control**.

## Features

### General syntax

Placeholder uses a clean, whitespace-significant syntax,
with an indentation style of your choosing. Tabs or 3 spaces?
Do whatever you want, as long as it's consistent within the file.

At a high level, a `.hldr` file with a single table and two records (one named, one anonymous) would essentially look like...

```
schema_name

  table_name

    record_name
      column1 value1
      column2 value2

    _ 
      column2 value3
```

... where records are grouped by table and tables are grouped by schema.

### Literal values

Currently, there are only literal values for booleans, numbers, and strings.

`hldr` currently parses all values as strings and passes them to Postgres
using the [simple query](https://www.postgresql.org/docs/current/protocol-flow.html#id-1.10.5.7.4)
protocol so that Postgres can convert values to their appropriate types.

**Important:** This means that `hldr` does not protect against SQL injection
from string values.

#### Booleans

Boolean values must be either `true` or `false`.
Unlike SQL, values like `TRUE` or `f` are not supported.

#### Numbers

Numbers can be integer or decimal values - `hldr` does not distinguish between
them or attempt to figure out their size.
They are passed as strings and Postgres coerces them to the right type
on a per-column basis.

#### Strings

Strings (single-quoted as in SQL) represent `char`, `varchar`, `text`, or
any other type such as arrays, timestamps, or even custom types that can
be represented as text.

For example, an array of integers would currently be written as `'{1, 2, 3}'`.

### Comments

Comments, like SQL, begin with `--` and can either be newline or trailing comments.

```
schema
  -- A newline comment
  table
    record
      column value -- A trailing comment
```

### Quoted identifiers

Schema, table, and column names must be double-quoted if they contain
non-standard characters like whitespace or punctuation.
Unlike in Postgres, however, identifiers are not automatically lowercased
or truncated by default, so Pascal- or camel-cased names can be written as-is.

```
schema

  some_table

    _
      -- Whitespace, etc. in a schema, table, or column name requires quoting
      "the answer to everything" 41

  "some ridiculous table-name"
    ...

  -- But special casing does not
  AnotherTable
    ...
```

### Named records

Records themselves can either be given a name, or they can be anonymous.
Naming records allows their columns (even those populated by the database
and not declared in the file) to be referenced in other records.

```
public
  person
    -- A named record
    kevin
      name 'Kevin'

    -- An anonymous record
    _
      name 'A Different Kevin'

  name
    -- Record names only need to be unique within
    -- the given table
    kevin
      value 'Kevin'
      origin 'Irish'
```

### References

Naming records allows them to be referenced elsewhere in the file,
whether referencing a declared column *or* a column
populated by default in the database.

There are several supported reference formats:

| Format | Example | When Allowed |
| --- | --: | --- |
| Fully-qualified | `schema.table@record.column` | Always |
| Table-qualified | `table@record.column` | When referencing a table in the same schema |
| Unqualified | `@record.column` | When referencing a record in the same table |

To demonstrate when the different formats are used:

```
schema1
  table1
    record1
      name 'Some record'

    record2
      -- Referencing record in same table can omit schema & table names
      name @record1.name

  table2
    record1
      -- Referencing record from another table in same schema
      -- can omit the schema name
      --
      -- Note that `id` is not specified in the record declaration but
      -- can still be referenced
      table1_id table1@record1.id

schema2
  table1
    _
      -- Must include schema and table when referencing a record from
      -- a table in another schema
      table2_id schema1.table2@record1.id
```

### Table aliases

Tables can also have aliases to help shorten qualified references,
and either the table name or alias can be used in any of the reference
formats as desired.

```
public
  person as p
    p1
      name 'Person 1'

  pet
    _
      -- Fully-qualified reference using alias
      person_id public.p@p1.id

    _
      -- Table-qualified reference using alias
      person_id p@p1.id

    _
      -- And the original table name can still be used
      person_id person@p1.id
```

## Planned features

See [enhancements](https://github.com/kevlarr/hldr/issues?q=is%3Aopen+is%3Aissue+label%3Aenhancement) for planned features.
