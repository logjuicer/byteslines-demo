# Copyright (C) 2022 Red Hat
# SPDX-License-Identifier: Apache-2.0

def logfile_iterator(reader):
    pos = 0
    while True:
        line = reader.readline()
        if not line:
            break
        line = line[:-1]
        pos += 1
        for subline in line.split("\\n"):
            yield (subline, pos)


if __name__ == "__main__":
    import sys
    count = 0
    for line in logfile_iterator(sys.stdin):
        count += len(line[0])
    print("Total:", count)
