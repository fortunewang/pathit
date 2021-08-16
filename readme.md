# pathit

## 1. iterate path

`pathit src`:

```
iterator.rs
main.rs
```

## 2. iterate path with hashing

`pathit --hash src`:

```
8df554d0a6c9a5457bf441e3ede6fdc06ee06e15cfa86ea69c058201c721642c, iterator.rs
b54f0ed2befc13e4844f32da47490a9e70d91f32ddb27ce2e8a1faabbc274cb1, main.rs
```

## 3. compare entries

`pathit ~/another/src | pathit-diff -f - src`:

```
+ iterator.rs
- another_mod/
- another_mod/mod.rs
```

where:

- `+`: entry exists in PATH only
- `-`: entry exists in reference(-d/--dir or -f/--file) only

## 4. compare entries and hashes

`pathit --hash ~/another/src | pathit-diff --hash -f - src`:

```
+ iterator.rs
x main.rs
- another_mod/
- another_mod/mod.rs
```

where:

- `+`: entry exists in PATH only
- `-`: entry exists in reference(-d/--dir or -f/--file) only
- `x`: entry exists in both PATH and reference, with different hashes
