# `dubble` A generic double-buffer

`dubble` is a crate which provides a generic implementation of double-buffering.


## What is double-buffering?

Typically in a game engine, there's a core loop which updates everything in the
game world each frame. However, this creates a small paradox. Let's say the
position of object B is dependent on the position of object A, but the position
of A is also somewhat dependent on the position of B. For example, consider a
simulation of a solar system, where gravitational forces between A and B affect
the motion of each other.

How do we decide which position to update first in the core game loop? The final
positions of A and B are going to depend on the order in which the objects are 
updated.

Double-buffering is a solution to this problem. We store two versions of A and
B. One version is for reading, and the other is for writing. When we update the
position of A, we read from the reading version of B, and store the new position
in the writing version of A. Likewise, when we update the position of B, we read
from the reading version of A and store the new position in the writing version
of B.

Finally, before the next iteration of the core game loop, the reading versions
of both A and B are updated with the values in their corresponding writing
versions.

Note that, when the position of A was updated, the reading version of A was not
changed, only the writing version. Likewise, when the position of B was updated,
we used the reading version of A, which was left unchanged when the position of
A was updated. Notice that now, the order in which things are updated doesn't
matter, since they both used the "old" version of the things that they depended
on.

This technique is called double-buffering, since it required two copies
(buffers) of the thing in question.


## Usage

### Initialisation

The most basic initialiser is `new()`, which initialises the read and write
versions of the object being double-buffered.

```rust
use dubble::DoubleBuffered;

// assuming you have a `Player` struct
let mut player = DoubleBuffered::new(Player::new());
```

There's also `construct_with()`, which accepts any `Fn() -> T` and uses that
to construct the read and write versions.

```rust
let mut player = DoubleBuffered::construct_with(Player::new);
```

Finally, `DoubleBuffered` also implements `Default` as long as the wrapped type
does, so you could also do

```rust
let mut player = DoubleBuffered::<Player>::default();
```


### Reading and writing

`read()` and `write()` are the basic methods which return references to the read
and write versions respectively. These references are immutable/mutable
respectively.

```rust
// reading
// read() -> &T
player.read().attack(monster);

// writing
// write() -> &mut T
player.write().dmg_hp(10.0);
```

But to make things easier, the buffer also implements `Deref<Target=T>` and
`DerefMut<Target=T>`, so the `read()` and `write()` calls can be omitted. The
trait implementations are simply wrappers around `read()` and `write()`
respectively.

```rust
player.attack(monster);
player.dmg_hp(10.0);
```


### Updating the read version with the write version.

`update()` will clone the write version onto the read version.

```rust
let mut player = DoubleBuffered::construct_with(Player::new);

// player starts with 100 hp
assert!(player.health == 100);

// do some damage to the player
player.dmg_hp(10);

// before the call to `update()`, the health will not have changed
assert!(player.health == 100);

// ... update other stuff ...

// now update the player's health
player.update();
assert!(player.health == 90);
```


### Usage with container types

`DoubleBuffered` implements `Index` and `IndexMut` so long as the wrapped type
does, so you can use a `DoubleBuffered<Vec<Actor>>` and update everything in
one call to `update()`.

```rust
let mut actors = DoubleBuffered::construct_with(Vec::<Actor>::new);
actors.push(the_player);
actors.push(monster_1);
actors.push(monster_2);
actors.push(monster_3);

// ... player and monsters update based on reads of the other's state ...

// update everyone
actors.update();
```

## Caveats

### Threading

Although the double-buffer itself can be sent across threads (it is `Send` and
`Sync` so long as `T` is as well), the *contents* cannot be accessed by other
threads without violating aliasing and mutability rules. Solution here for now
is to wrap this type in a `Mutex`.

