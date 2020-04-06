# Cedar
Cedar (pronounced sēʹdər or see-der) is a general purpose programming language
designed to build tools or scrips where one doesn't need speed, but wants
something small and easy to maintain, while still providing 1st class tooling
and language features to make that easier to do.

## Installing Rust
The bytecode interpreter is built with Rust. In order to build the project you
will need to install the compiler on your system. More instructions can [be found
here](https://www.rust-lang.org/tools/install)

## Learning Cedar

Cedar is a fairly simple language to learn and get started using.

### Types

Cedar has numbers (currently only 64 bit Floating Point):

```
// Examples of valid number literals
12.0
12
```

Booleans:

```
// Examples of valid boolean literals
true
false
```

and Strings:
```
// Examples of valid String literals
"Hello, World!"
```

### Comments

All comments start with // and will ignore everything to the end of the line

```
// This line won't be parsed.
let x = 10; // Everything pass the // to the newline won't be parsed
```

### Variables

You can assign values to variables like so:

```
let x = 10;
```

You can reassign values to variables:

```
x = 20;
```

You can also use them as part of statements;

```
// This will print 20
print x;
```

Valid identifiers are ASCII Alphanumeric characters in kebab-case:

```
let valid-variable = 0;
let not_valid = 1;
```

### Scopes

Cedar also has scopes which allows for shadowing:

```
let x = 10;
{
  let x = 20;
  // Will print 20
  print x;
}
// Will print 10
print x;
```

These can be nested as well:

```
{
  {
    {
      print "Three scopes deep";
    }
  }
}
```

### Functions

Cedar also has functions:

```
fn foo() {
  let a = 5;
  let b = 6;
  print a + b;
}
```

You return values and invoke functions like so:

```
fn foo() {
  let a = 5;
  let b = 6;
  return a + b;
}

print foo();
```

Functions can also take parameters:

```
fn hello(first-name, last-name) {
  print "Hello there " + first-name + " " + last-name + "!";
}

hello("Michael", "Gattozzi");
```

### Operations

You can concatenate values onto strings with `+`:

```
let the-answer = 42;
print "The answer to life, the universe, and everything is: " + the-answer;
```

You can also add, subtract, multiply and divide numbers using the `+`, `-`, `*`
and `/` operators in that order:

```
let a = 10 + 5; // 15
let b = 10 - 5; // 5
let c = 10 * 5; // 50
let d = 10 / 5; // 2
```

You can negate numbers like so:

```
let x = -10;
print x; // prints -10
```

You can also do comparisons using greater than (`>`), less than (`<`), equal
(`==`), not equal (`!=`), greater than or equal (`>=`) or less than or equal
(`<=`) operators. These can only be used between values of the same type:

```
print true != false; // true
print 1 < 2; // true
print 2 > 1; // true
print 2 >= 1; // true
print 2 <= 1; // false
print "Hello!" == "Hello!"; // true
```

### Control Flow
Lastly Cedar has constructs for control flow. For instance it has if and if else
statements:

```
let x = 10;
if x < 20 {
  print true;
}

if x != 10 {
  print "x is not equal to 10!";
} else {
  print "X is equal to 10!";
}
```

Cedar also has for and while loops:

```
let x = 0;
while x < 10 {
  print "while loop";
  x = x + 1;
}

for let y = 0; y < 10; y = y + 1 {
  print "for loop";
}
```

### Native Functions

Cedar also has functions that allow you to interact with the operating system
granting you I/O capabilities, manipulating values, and more. All of these
functions live inside of `src/libstd` and you should take the time to peruse
them to see what cedar is capable of beyond the basic syntax.

## Current State

Cedar is in relatively early stages of development. As a result breaking changes
will likely happen often and your code is therefore likely to break. As long as
you don't upgrade your interpreter this shouldn't affect you, but feedback of
new features is greatly appreciated.

## Contributing

Do you like Cedar? Want to help out? Great! Please take a look at our
[CONTRIBUTING.md](CONTRIBUTING.md) file for more information.

## Code of Conduct

We have a strictly enforced [Code of Conduct](CODE_OF_CONDUCT.md) that all
participants in the project will be held to whether on the repo or in places
pertaining to the project. Continued failure to uphold the CoC or in the case of
more egregious cases will result in a permanent ban. If you wish to see the
reasoning behind this please [look at the commit][COC] that introduced the CoC.

## Licensing

All code is licensed the Apache-2.0 license. Any contributions to the project
will be licensed under the same terms. See the [LICENSE](LICENSE) file for more
details.

[COC]: https://github.com/mgattozzi/cedar/commit/fda64e1baeea90acfdc8e85b751fb1659819753b
