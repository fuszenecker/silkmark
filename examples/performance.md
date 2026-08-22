# Large document performance sample

This document is intended for exercising SilkMark's bounded image queue and render statistics.

## Image queue

When opened over HTTPS, SilkMark v0.38 runs no more than four image downloads concurrently.

![one](https://dummyimage.com/640x120/eeeeee/111111.jpg&text=one)
![two](https://dummyimage.com/640x120/eeeeee/111111.jpg&text=two)
![three](https://dummyimage.com/640x120/eeeeee/111111.jpg&text=three)
![four](https://dummyimage.com/640x120/eeeeee/111111.jpg&text=four)
![five](https://dummyimage.com/640x120/eeeeee/111111.jpg&text=five)
![six](https://dummyimage.com/640x120/eeeeee/111111.jpg&text=six)

## Long section

The renderer reports parse and GTK construction timing with `--stats`.

```sh
silkmark --stats -v https://example.org/large.md
```
