# Placeholder

*Easy, declarative data seeding for PostgreSQL*

Placeholder strives to make generating test data much more succinct
and cleaner than using SQL, PL/pgSQL, or even other programming languages
with dedicated factory libraries, like Python and FactoryBot.

With some powerful, easy to read syntax and a simple `hldr` command,
you can have a well populated database in no time without setting up
languages, dependencies, or verbose factory classes.

## Planned Features

### Simple records

Singular records can easily be created, and they can be named or anonymous.

```
public
  person -- Table name
    stacey:
      name 'Stacey'

    _:
      name 'Kevin'
      age 39

-- For schema and/or table names with whitespace, use double quotes
"schema name with spaces"
  -- Bare identifiers are not converted to lowercase as they are in SQL
  SomeTable
    _:
      id 1
```

### References

Named records are useful because they can be referenced elsewhere,
particularly in foreign keys. Creating a record returns all columns,
not just those declared in the file, so it lets you reference columns
that are populated with database-side defaults.

You can reference a column from a named record using the
fully-qualified format `schema.table#name.column`.

- The `schema` can be omitted if the record being created is in the same
scheme as the table being referenced
- The `table` can be omitted


```
public:
  person:
    stacey:
      name 'Stacey'
      likes_pizza true

    kevin:
      name 'Kevin'
      likes_pizza #stacey.likes_pizza

  pet:
    cupid:
      name 'Cupid'
      person_id person#stacey.id
```

There are some shorthand forms that remove some of the redundancy
of declarations like `likes_pizza #stacey.likes_pizza` and
`person_id person#stacey.id` when the column names are identical.

- The `[column] #ref` pattern looks for a named reference
in the same table and uses its value from the same column

- The `[table.column] #ref` pattern is useful for foreign key
columns whose names perfectly match the referenced table
and column. *(Note: This currently doesn't support cross-schema references.)*

For instance, the above can also be written as:

```
public:
  person:
    stacey:
      likes_pizza true

    kevin:
      [likes_pizza] #stacey

  pet:
    cupid:
      [person.id] #stacey
```

### Composition

Writing every column value manually can be tedious, especially if
numerous records have similar sets of values or we want to reference
multiple values from another record.

We can compose records in several ways.

#### Copy values from one record into another

All values can be copied from one record into another,
and it's possible to override any that need to be changed.
(**Note:** This will only copy fields explicitly declared in the
original reference - defaults or generated columns will not be copied.)

```
public:
  pet:
    cupid:
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
public:
  pet:
    cupid:
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
This is where **templates** are useful, and they can be declared at any
scope and define any set of columns.

A record can use values from multiple templates, and any templates used
must be accessible in scope to that table *and* have the right columns.

```
-- This template can be used in any schema or table,
-- as long as the table has the 'color' column
$brown:
  color 'brown'

public:
  -- This template can be used in any 'public' schema table
  $friendly:
    friendly true

  pet:
    -- This template can only be used by 'pet' records
    $cat:
      species 'cat'

    cupid: $friendly $brown $cat
      name 'Cupid'

    eiyre: $friendly $cat
```

Additionally, templates and record copying can be intermixed in any order.

```
public:
  pet:
    $black:
      color 'black'

    $friendly:
      friendly true

    $cat:
      species 'cat'

    cupid: $friendly $cat
      name 'Cupid'
      sex 'male'

    eiyre: #cupid $black
```

### Default values

Table-level defaults are easy to define and override.

```
public:
  pet:
    @species 'cat'

    cupid:
      name 'Cupid'

    eiyre:
      name 'Eiyre'

    huxley:
      name 'Huxley'
      species 'dog'
```

### Series

It can be useful to generate series of records, either a certain number
or based on a list of values.
These series can also be anonymous or named, and they can also
be composed from other records or templates like singular records can.

```
public:
  pet:
    $cat
      species 'cat'

    *5n:
      name 'Pet ${n}'

    *10n cats: $cat
      name 'Cat ${n}'

    *each-name ['Cupid' 'Eiyre'] other_cats:
      name name

    another_cat:
      [name] #cats[0]
```
