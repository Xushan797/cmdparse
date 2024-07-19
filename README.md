# cmdparse

```sh
echo 'sh -c "mycmd -f && yourcmd --bar=foo || bash -c \"ourcmd baz\" --bashflag" |& {quux|corge;}' | cmdparse
mycmd -f
yourcmd --bar=foo
ourcmd baz
quux
corge
```
