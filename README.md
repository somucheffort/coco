# coco

coco is a dynamic-typed language similar to js and kotlin 

## example

see [example folder](./example) for more advanced examples

```coco
// assume we have one coco
let coco = 1

while (coco < 67) {
    // we would add 1 coco until we have 67
    coco += 1
}

log('total cocos:', coco)
```

# running

currently, it does not support repl, so only way to execute any coco is to run via file

```bash
$ cargo run filename.coco
```

# contributing

all contributions are welcome, if they are aimed on making language better in any kind

# license

[MIT](./LICENSE)