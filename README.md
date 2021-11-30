# Placeholder

*Easy, declarative data seeding for PostgreSQL*

Placeholder strives to make generating test data much more succinct
and cleaner than using SQL, PL/pgSQL, or even other programming languages
with dedicated factory libraries, like Python and FactoryBot.

With some powerful, easy to read syntax and a single `hldr` command,
you can have a well populated database in no time without setting up
languages, dependencies, or verbose factory classes.

## Usage

Placeholder currently must be compiled from source.

Once compiled and installed to your path,
to run the command you must specify the file to load and the
database connection string.

```
# If installed in path as `hldr`
$ hldr -f path/to/data/file -d postgres://user:password@host:port/db
```

By default, Placeholder will roll back the transaction,
which is useful to test that all records can be created.
If you want to commit the records, pass the `--commit` flag.

```
$ hldr -f path/to/data/file -d postgres://user:password@host:port/db --commit
```

## Features

Placeholder uses a clean, whitespace-significant syntax,
with an indentation style of your choosing. Tabs or 3 spaces?
Do whatever you want, as long as it's consistent.

Records themselves can either be given a name, or they can be anonymous.
Named records are not useful yet but they [will be soon](#reference-values).

There are literal values for booleans, numbers, and text strings.

- Numbers can be integers or floats; it will be up to the database to
coerce them to the right type based on the column.

- Text strings will also be coerced, which means they can be
used to represent `varchar`, `text`, `timestamptz`, arrays like `int[]`, or any
other type (even user-defined types) that can be constructed in SQL from string literals.

The general file format looks like...

```
schema
  table
    record
      column value
```
... where any number of schemas, tables, records, and attributes can be defined.

For example, a simple file that looks like...

```
public
  person
    fry
      name 'Philip J. Fry'
      hair_color 'red'

    leela
      name 'Turanga Leela'

  pet
    _
      name 'Nibbler'
```

... will create three record (two named, one anonymous) like:

```sql
INSERT INTO "public"."person" ("name", "hair_color")
  VALUES ('Philip J. Fry', 'red');

INSERT INTO "public"."person" ("name")
  VALUES ('Turanga Leela');

INSERT INTO "public"."pet" ("name")
  VALUES ('Nibbler');
```

Comments, like SQL, begin with `--` and can either be in their own line or inline.

```
public
  -- This table has people in it
  person
    fry -- This is a named record
      name 'Philip J. Fry'

    -- This is an anonymous record...
    _
      -- ... even though we know its name
      name 'Morbo'
```

Bare identifiers (ie. `public`, `person`, and `name` in the example above)
are not lowercased or truncated automatically, like in SQL.
Statements use quoted identifiers automatically,
but (for the sake of the parser) you must explicitly quote identifiers
that have whitespace, punctuation, etc.

```
"schema with whitespace"
  "table.with -- dashes"
    my_record
      "column with spaces" 42
```



## Planned features

### Easier command options

This is a lot to type:

```
$ hldr -f path/to/data/file -d postgres://user:password@host:port/db --commit
```

A nice-to-have would be to look for default files in the current directory
to avoid all that typing, eg. a `data.hldr` file for the records and
a `hldr.env` file with the database connection string.

### Reference values

Named records should be referenceable elsewhere.
Creating a record should return all columns, not just those declared
in the file, so they should let you reference columns that are populated
with database-side defaults.

Proposed is a format that lets one reference a column from a named
record using the fully-qualified format `schema.table#name.column`
with several shorthand varieties:

- The `schema` can be omitted if the record being created is in the same
scheme as the referenced table and record
- The `table` can be omitted if the record being created is in the same table as the referenced record

For example:

```
schema1
  person
    fry
      name 'Philip J. Fry'
      likes_pizza true

    leela
      name 'Turanga Leela'
      likes_pizza #fry.likes_pizza

  pet
    _
      name 'Nibbler'
      person_id person#leela.id

schema2
  robot
    _
      name 'Bender Bending Rodriguez'
      lives_with schema1.person#fry.id
```

Some additional shorthand forms could remove some of the redundancy
of declarations like `likes_pizza #fry.likes_pizza` and
`person_id person#leela.id` when the column names are identical.

- The `[column] #ref` pattern looks for a named reference
in the same table and uses its value from the same column

- The `[table.column] #ref` pattern is useful for foreign key
columns whose names perfectly match the referenced table
and column.
*(Note: This would not support cross-schema references.)*

For instance, the above can also be written as:

```
schema1
  person
    fry
      name 'Philip J. Fry'
      likes_pizza true

    leela
      name 'Turanga Leela'
      -- Expands to `likes_pizza #fry.likes_pizza`
      [likes_pizza] #fry

  pet
    _
      name 'Nibbler'
      -- Expands to `person_id person#leela.id`
      [person.id] #leela

schema2
  robot
    _
      name 'Bender Bending Rodriguez'
      -- There's no shortening this :*(
      lives_with schema1.person#fry.id
```

### Composition

Writing every column value manually can be tedious, especially if
numerous records have similar sets of values or we want to reference
multiple values from another record.

We should be able to compose records in several ways.

#### Copy values from one record into another

All values are copyable from one record into another,
and it's possible to override any that need to be changed.
(**Note:** This would only copy fields explicitly declared in the
original reference - defaults or generated columns will not be copied.)

```
public
  pet
    cupid
      friendly true
      name 'Cupid'
      sex 'male'
      species 'cat'

    eiyre: #cupid
      name 'Eiyre'
```

If desired, only a subset of values can be copied,
which is useful when columns in the second record are wanted
to be null.

```
public
  pet
    cupid
      friendly true
      name 'Cupid'
      sex 'male'
      species 'cat'

    eiyre: #cupid[friendly species]
      name 'Eiyre'
```

#### Define a template to use when populating records

Sometimes we want common sets of attributes to apply to multiple records,
but we don't want them to come from a record itself.
This is where **templates** would be useful,
and they should be declareable at any
scope and define any set of columns.

A record could use values from multiple templates, and any templates used
must be accessible in scope to that table *and* have the right columns.

```
-- This template can be used in any schema or table,
-- as long as the table has the 'color' column
$brown
  color 'brown'

public
  -- This template can be used in any 'public' schema table
  $friendly
    friendly true

  pet
    -- This template can only be used by 'pet' records
    $cat
      species 'cat'

    cupid: $friendly $brown $cat
      name 'Cupid'

    eiyre: $friendly $cat
```

Additionally, templates and record copying should be able to be
intermixed in any order.

```
public
  pet
    $black
      color 'black'

    $friendly
      friendly true

    $cat
      species 'cat'

    cupid: $friendly $cat
      name 'Cupid'
      sex 'male'

    eiyre: #cupid $black
```

### Default values

Table-level defaults should be easy to define and override.

```
public
  pet
    @species 'cat'

    cupid
      name 'Cupid'

    eiyre
      name 'Eiyre'

    huxley
      name 'Huxley'
      species 'dog'
```

### Series

It can be useful to generate series of records.
These series should also be anonymous or named, and they should also
be able to be composed from other records or templates like singular records can.

```
public
  pet
    $cat
      species 'cat'

    -- An anonymous series
    *5n
      -- Some form of interpolation should be possible
      name 'Pet ${n}'

    -- A named series with composition
    *10x cats: $cat
      name 'Cat ${x}'

    yet_another_cat
      [name] #cats{0}
```

Additionally, would series from lists of values be desirable?

```
public
  pet
    $cat
      species 'cat'

    *{'Cupid' 'Eiyre'}n cats: $cat
      name n

    eiyre2: #cats{1}[name]
```
