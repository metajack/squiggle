# throttle api use (jack)

this is set to 3.5s per request, which is crappy an often fails anyway

# handle bad guesses

these send back failing cases or something. we should add these to the
constraints so the next guess will be better

# constrain gen to use only one fold (eatkinson)

also, tfold means the fold is the top level

# buffer guesses

guesses should continue to be generated in case more are needed. at least up
to some cache size. these will need to be thrown away if the guess is right.

# parallelize problems or handle multiple problems at once

i think teh former is better than the latter if we can do it
