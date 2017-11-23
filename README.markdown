# Tools for assembling and disassembling Prehistorik Man (Game Boy)

## prem-unpack

Unpacks a compressed Prehistorik Man resource.

Example (extracting and unpacking the font):

```shell
dd bs=1 if=prehistorik_man.gb skip=122925 count=405 | prem-unpack -o font.bin
```

# License and copyright

Licensed under the MIT license.
Copyright (C) 2017 Joonas Javanainen <joonas.javanainen@gmail.com>
