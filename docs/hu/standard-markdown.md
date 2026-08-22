# Alap, hordozható Markdown

Ezek a formák alkotják a SilkMarkban ajánlott hordozható alapot. Más modern Markdown megjelenítőkben is jellemzően működnek.

## Címsorok

```md
# H1
## H2
### H3
#### H4
##### H5
###### H6
```

Setext H1/H2:

```md
H1 címsor
=========

H2 címsor
---------
```

## Bekezdések és sortörés

```md
Első bekezdés.

Második bekezdés.
```

Egy normál sortörés egy bekezdésen belül soft break. Két szóköz a sor végén hard break:

```md
Első sor.  
Második sor.
```

## Kiemelés

```md
*dőlt*
**félkövér**
**félkövér és *dőlt***
```

## Inline kód

```md
A `cargo build --release` parancs fordít.
```

Több backtick használható, ha a tartalom maga is backticket tartalmaz:

````md
``egy ` karakter``
````

## Kódblokkok

````md
```rust
fn main() {
    println!("Hello");
}
```
````

Tilde fence is használható:

```md
~~~text
Szövegblokk
~~~
```


### Behúzott kódblokk

A CommonMark szerinti, négy oszloppal behúzott kódblokk is támogatott:

```md
    fn main() {
        println!("Hello");
    }
```

Új dokumentumokban a nyelvmegjelölés és az egyértelműség miatt továbbra is a fenced code block ajánlott.

## Idézet

```md
> Idézet.
>
> Második bekezdés.
```

Egymásba ágyazva:

```md
> Külső
>> Belső
```

## Rendezetlen lista

```md
- alma
- körte
- szilva
```

A `*` és `+` marker is elfogadott.

## Rendezett lista

```md
1. első
2. második
3. harmadik
```

A `)` végű CommonMark lista-marker is támogatott:

```md
1) első
2) második
```

A megadott kezdő sorszám megmarad:

```md
7. hetedik
8. nyolcadik
```

## Egymásba ágyazott blokkok

```md
- fő elem
  - második szint
    1. harmadik szint
```

Listaelemen belül idézet és fenced code block is használható.

## Vízszintes elválasztó

```md
---
```

Támogatott még például `***`, `___` és `- - -`.

## Linkek

```md
[Dokumentáció](guide.md)
[Rust](https://www.rust-lang.org/)
[Rust](https://www.rust-lang.org/ "Rust nyelv")
[Telepítés](#telepítés)
[API / Típusok](api.md#típusok)
```

## Referencia-linkek

```md
A [Rust][rust] nyelv.

[rust]: https://www.rust-lang.org/ "Rust"
```

Rövid formák:

```md
[Dokumentáció][]
[Dokumentáció]: docs/index.md

[Home]
[Home]: index.md
```

## Képek

```md
![Logó](images/logo.png)
![Külső kép](https://example.org/image.png)
```

Referencia-kép:

```md
![Logó][logo]

[logo]: images/logo.png "Projektlogó"
```

## Autolink

URI autolink:

```md
<https://example.org/>
```

Email autolink:

```md
<user@example.com>
```

## Escape

Markdown írásjelek literálisan backslash segítségével:

```md
\*nem dőlt\*
\[nem link\]
```

## HTML entity

```md
&amp;
&lt;
&gt;
&#169;
&#x1F642;
```

## Megjegyzés a kompatibilitásról

Ez a fejezet a SilkMarkban stabilan használható, hordozható Markdown-formákat dokumentálja. A SilkMark célja a CommonMark/GFM gyakorlati kompatibilitás, nem a specifikáció minden mesterséges vagy ritka parser-edge-case-ének külön támogatása.

## CommonMark 1.0 kiadási megjegyzések

A SilkMark kezeli a `*`/`**` és `_`/`__` kiemelési formákat, a CommonMark ASCII
backslash escape készletét, az URI/e-mail autolinkeket, a behúzott kódot, valamint
a fenced code blokkok szabványos, legfeljebb három kezdő szóközét. A named
karakterreferenciákhoz a CommonMark által használt teljes, pontosvesszővel zárt
HTML5 névkészlet be van építve. A raw HTML továbbra is szándékosan inert szöveg.
