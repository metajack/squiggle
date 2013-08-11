# benchmark the parallelism

see if it's faster and how much so

# prevent gen of stupid expressions like (or 0 1) and things.

# buffer guesses

guesses should continue to be generated in case more are needed. at least up
to some cache size. these will need to be thrown away if the guess is right.
