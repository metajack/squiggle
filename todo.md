# constrain gen to use only one fold (eatkinson)

also, tfold means the fold is the top level

# buffer guesses

guesses should continue to be generated in case more are needed. at least up
to some cache size. these will need to be thrown away if the guess is right.

# parallelize problems or handle multiple problems at once

i think teh former is better than the latter if we can do it

# ./squiggle faketrain

generate random training problems for benchmarking, testing so we don't have
to use the crappy webapi witih throttling
