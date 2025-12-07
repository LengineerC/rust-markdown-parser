# Refactored *Markdown*
## Multiline Test

This **is** line 1.
This is *line **2** (should be same paragraph)*.

This is ***bold and italic*** text.
This is line 3 *(new paragraph).
![Rust Logo](https://www.rust-lang.org/logos/rust-logo-512x512.png)

test ****ds****

And here is an image 0啊啊啊啊啊啊啊啊啊嗷噢噢噢噢噢噢噢噢噢o:
Links can also contain **bold text**: [Check **this** out!](https://example.com).
-----------
Test ~~edge~~ cases:
* Not a link: [broken] (link)
* Not an image: ! [space matters] (url)
**
> Level 1 BlockQuote
>
> > Level 2 Nested BlockQuote
> > containing **bold** text.
>
> Back to Level 1.
****
# Code Test

Here is some inline code: `let x = 5;` and `Option<T>`.
And here is a block:
___

```rust
fn main() {
    println!("Hello, world!");
    if 1 < 2 {
        // indent check
    }
}
Check html escape: <div> inside code.
```

# Del Test
This is ~~wrong~~ correct text.
This is ~~*wrong*~~ correct text.
This is ~~**bold** and deleted~~.
Edge case: ~single tilde~ (not deleted).

# List Test

Unordered List:
- Item 1
- Item 2 with **bold**
* Item 3 (star)
+ Item 4 (plus)

Ordered List:
1. First step
2. Second step
3. Step with ~~strike~~
7. Step with ~~strike~~

Mixed (Should break list):
1. Item B (New list)
2. Item B (New list)
- Item A
1. Item C (New list)

# Nested List Test

- Level 1 Item A
- Level 1 Item B
  - Level 2 Item A
  - Level 2 Item B
    - Level 3 Item
  - Level 2 Item C
- Level 1 Item C

Ordered Nesting:
1. Step 1
   1. Sub-step 1.1
   3. Sub-step 1.2
2. Step 2
# Table Test

| Name    |  Age  |       Role | Gender |
| :------ | :---: | ---------: | ------ |
| Alice   |  24   |    **Dev** | F      |
| Bob     |  30   |    Manager | M      |
| Charlie |  28   | *Designer* | M      |

# Escape & HTML Test

## 1. Escaping
* This is a bullet.
\* This is NOT a bullet, it's a literal star.

**Bold** vs \*\*Not Bold\*\*.

Markdown chars: \[link\] without link, \# not a header.

## 2. Raw HTML
This is text. <br> This is a new line.
<span style="color: red">Red Text</span>.
<!-- test -->
<div>
  Content inside div (treated as inline html here).
</div>

Comparison:
* 1 < 2 (Should be escaped as &lt;)
* <br> (Should be raw HTML)
