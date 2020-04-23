Contributing to the game is easy! you don't even have to know how to code!

If you have an idea for a new mob or a new item, you can add it to the game by changing the config files.
If you look in the config folder, you should be able to see mobs.txt.
To make a new mob, open the mobs file and scroll to the bottom. Then, add something like this to the end of the file:

```
/begin/
String name  Ben-the-Dragon-at-the-End
String[] entrance  "How-dare-you-disturb-my-slumber!"
String[] attack  *Leave-at-once!* *RAAAAAAWWWWRRRRR!!!!*
String[] player-run  "At-least-you-have-some-concience"
String[] player-victory  *How-could-a-mere-mortal...*
String[] mob-victory  *Puny-Mortal*
int aggression  100
int dmg  12
int xp  350
int health  3
String[] drops "Dragon-scales" "Dragon-heart" "Eternal-embers"
int drop-min  0
int drop-max  3
int speed 8
LongString img
             __.-/|
             \`o_O'
              =( )=  +-----+
                U|   | BEN |
      /\  /\   / |   +-----+
     ) /^\) ^\/ _)\     |
     )   /^\/   _) \    |
     )   _ /  / _)  \___|_
 /\  )/\/ ||  | )_)\___,|))
<  >      |(,,) )__)    |
 ||      /    \)___)\
 | \____(      )___) )____
  \______(_______;;;)__;;;)
  /end/
  ```
and Hey presto! that mob is in the game!
basically, the way this works is that each line is read as a property that the mob has. So `int dmg 12` means that this mob deals 12 damage per turn.
the first part each line tells me what type of thing to expect. so int -> integer, String -> text, String[] -> multiple peices of text, etc.

The one exception to this is the `LongString img` type. You can only have one `LongString` per mob, and it should always be the last thing in the mob definition.

you don't have to have every property listed above. The game can make assumptions for most of the values. For example, you could just not write any quotes and not have an image for your mob.
the basic minimum requirements for a mob are:
name
dmg
xp
health
speed
if you have at least these, you're good!
