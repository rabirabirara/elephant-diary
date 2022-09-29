# elephant-diary
A great listener who remembers everything you say.

# Motivation

EVERYONE has a spam account.  Whether it be on INSTAGRAM, or DISCORD, or something...  Everyone's gotta vent their thoughts somewhere.

I happen to do that on Discord.  My Discord server has a channel where I just talk about anything I want.

So, why don't I just... spam an actual file, instead of telling all my friends all the crazy thoughts and manga panels and game rants?

#### Ever wanted a lifeless partner who says nothing

# Usage

Install with cargo and do `cargo run`.  Should be easy if you know what it is.  If you don't, well.  I don't know what to say.  
Install cargo and go learn Rust.

# Goals

- [x] Basic editing
- [x] Basic app logic
- [x] File saving and MRU
- [ ] Elephant speaks back, occasionally affirming you, telling you are valid, or just emoting like "mhm" and "right"; maybe in the message bar?
or maybe a little corner of the message view?
  - can select mood of the elephant?
- [ ] Proper editing: cursors, input movement, motions/actions
  - already got a nice gapbuffer, which is cool.  but a `Vec<String>` might be better for an editor which needs to go from line to line
- [ ] Mouse support, scrollbars
- [ ] Image support, either inline or popup (dependant on Rust libraries, e.g. sixel bindings)
- [ ] Markdown parsing - i.e. italics, bold, code highlighting
- [ ] Maybe a Discord client in the future...?
