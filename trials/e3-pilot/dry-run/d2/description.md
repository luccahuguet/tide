Split `record_hit` into a read of the current count and a write of the
count plus one, so that each critical section is as short as possible.
