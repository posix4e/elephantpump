import sys
lines_seen = set() # holds lines already seen
for line in sys.stdin:
    if not line.strip().endswith(';'):
	print line
    elif not "extern int" in line or line not in lines_seen: # not a duplicate
	print line
        lines_seen.add(line)
