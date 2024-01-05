# Turing Machine Solver

I got the board game Turing Machine for Christmas, so naturally, I had to write
a solver for it. This solver was mainly meant as a fun coding exercise for me,
but maybe others will get some use out of it.

## Usage

Apart from the solver, you will need a physical version of the game, as the
solver does not know the verification cards.

Run the solver from the command line using the syntax

```
tm_solver A B C D ...
```

where A, B, C, D, ... are the numbers of the challenge's criteria cards, e. g.

```
tm_solver 4 9 11 14
```

solves the first challenge in the manual. Use an additional `-v` flag to print
more verbose information about the unique solutions the puzzle has and the
resulting solution tree.

Once the solution tree is constructed, the program will ask you to perform
various tests with a given combination. Once it is confident that it knows the
correct combination, it will print the answer to the console.

## How it works

It should go without saying, but this section contains SPOILERS about the game.
You might want to try to figure out some stuff yourself, first.

The program first generates all possible input codes and runs them through all
possible rule combinations. Utilizing the fact that some combination of rules
make it impossible to distinguish two or more codes, it can rule out certain
rule combinations right away.

In the next step, the software constructs an optimal binary search tree to find
the correct combination of critera in the smallest amount of tests. Note that
this minimizes both average and worst-case scenarios, but it's still possible
to get lucky and beat the solver if you get lucky.

## Limitations

Currently only rules 1-25 are implemented, and extreme and nightmare modes are
not supported. All of them seem to be the same basic problem, which I have an
idea about how to implement, but it will take a little work.